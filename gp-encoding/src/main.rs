// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use geoplegma::types::{DggrsUid, Point, RefinementLevel};
use gp_encoding::{
    Compression, StorageBackend, ZarrBackend, convert_geotiff_file_to_backend, format_value,
    query_value_for_point,
};

#[derive(Parser, Debug)]
#[command(
    name = "gp-encoding",
    about = "CLI for querying and encoding DGGS-backed datasets",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert a GeoTIFF file into a Zarr-backed encoded dataset.
    ConvertGeotiff(ConvertGeotiffArgs),
    /// Add a coarser resolution level by aggregating an existing level.
    AddLevel(AddLevelArgs),
    /// Query a value at geographic coordinates.
    Query(QueryArgs),
    /// Print summary statistics for an encoded Zarr store.
    Stats(StatsArgs),
}

#[derive(Args, Debug)]
struct ConvertGeotiffArgs {
    /// DGGRS to use for the output dataset.
    #[arg(short, long)]
    dggrs: DggrsUid,
    /// Input GeoTIFF path.
    #[arg(short, long)]
    input: PathBuf,
    /// Output Zarr store path.
    #[arg(short, long, default_value = "./tmp/gp_encoding_geotiff_convert")]
    output: PathBuf,
    /// Optional compression for Zarr chunks.
    #[arg(long, value_enum)]
    compression: Option<Compression>,
    /// Print conversion statistics after a successful conversion.
    #[arg(long)]
    report: bool,
}

#[derive(Args, Debug)]
struct QueryArgs {
    /// Zarr store path.
    #[arg(short, long)]
    store: PathBuf,
    /// Refinement level.
    #[arg(short, long)]
    level: u8,
    /// Longitude in degrees.
    #[arg(long)]
    lon: f64,
    /// Latitude in degrees.
    #[arg(long)]
    lat: f64,
    /// Optional band index (0-based). If omitted, all bands are queried.
    #[arg(long)]
    band: Option<u32>,
}

#[derive(Args, Debug)]
struct StatsArgs {
    /// Zarr store path.
    #[arg(short, long)]
    store: PathBuf,
}

#[derive(Args, Debug)]
struct AddLevelArgs {
    /// Zarr store path.
    #[arg(short, long)]
    store: PathBuf,
    /// Source level to aggregate from (defaults to the highest level in the store).
    #[arg(long)]
    source_level: Option<u8>,
    /// Target level to create (must be lower than the source level).
    #[arg(long)]
    target_level: u8,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::ConvertGeotiff(args) => run_convert_geotiff(args),
        Commands::AddLevel(args) => run_add_level(args),
        Commands::Query(args) => run_query(args),
        Commands::Stats(args) => run_stats(args),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_convert_geotiff(args: ConvertGeotiffArgs) -> Result<(), String> {
    if args.output.exists() {
        std::fs::remove_dir_all(&args.output).map_err(|e| {
            format!(
                "failed to clean output store {}: {e}",
                args.output.display()
            )
        })?;
    }

    let (backend, source_report, conversion_report) =
        convert_geotiff_file_to_backend::<ZarrBackend>(
            &args.input,
            &args.output,
            args.dggrs,
            args.compression,
        )
        .map_err(|e| e.to_string())?;

    println!("Conversion successful");
    println!("  Input:      {}", args.input.display());
    println!("  Output:     {}", args.output.display());
    println!("  Levels:     {:?}", backend.levels());

    if args.report {
        print!("{source_report}");
        print!("{conversion_report}");
    }

    Ok(())
}

fn run_query(args: QueryArgs) -> Result<(), String> {
    let backend = ZarrBackend::open(&args.store).map_err(|e| e.to_string())?;

    let refinement = RefinementLevel::from(args.level);
    let point = Point::new(args.lat, args.lon);

    let band_count = backend.band_count();
    let bands: Vec<u32> = if let Some(band) = args.band {
        if band >= band_count {
            return Err(format!(
                "band {band} is out of range (store has {band_count} bands)"
            ));
        }
        vec![band]
    } else {
        (0..band_count).collect()
    };

    println!(
        "Querying point ({}, {}) at level {}",
        args.lon, args.lat, args.level
    );

    for band in bands {
        let value_bytes = query_value_for_point(&backend, refinement, band, point)
            .map_err(|e| format!("query failed for band {band}: {e}"))?;
        let dtype = &backend.metadata().attributes[band as usize].dtype;
        let formatted = format_value(dtype, &value_bytes)
            .map_err(|e| format!("failed to decode value for band {band}: {e}"))?;
        println!("  band {band} ({dtype:?}): {formatted}");
    }

    Ok(())
}

fn run_stats(args: StatsArgs) -> Result<(), String> {
    let backend = ZarrBackend::open(&args.store).map_err(|e| e.to_string())?;
    let metadata = backend.metadata();

    println!("Store:       {}", args.store.display());
    println!("DGGRS:       {}", metadata.dggrs);
    println!("Chunk size:  {}", metadata.chunk_size);
    println!("Levels:      {:?}", backend.levels());
    println!("Bands:       {}", metadata.attributes.len());

    let levels = backend.levels();
    let band_count = backend.band_count();

    for level in levels {
        let num_chunks = backend.num_chunks(level).map_err(|e| e.to_string())?;
        let (_, chunk_ids) = backend
            .chunk_ids_for_level(level)
            .map_err(|e| e.to_string())?;
        let chunk_ids_len = chunk_ids.len();
        println!("\nLevel {level}");
        println!("  Logical chunk count: {num_chunks}");
        println!("  Stored chunk IDs   : {chunk_ids_len}");

        for band in 0..band_count {
            let dtype = &metadata.attributes[band as usize].dtype;

            println!("  Band {band} ({dtype:?})");
        }
    }

    Ok(())
}

fn run_add_level(args: AddLevelArgs) -> Result<(), String> {
    let mut backend = ZarrBackend::open(&args.store).map_err(|e| e.to_string())?;
    let source_level = if let Some(source) = args.source_level {
        source as u32
    } else {
        backend
            .levels()
            .into_iter()
            .max()
            .ok_or_else(|| "store has no levels".to_string())?
    };

    backend
        .add_level_from_existing(source_level, args.target_level as u32)
        .map_err(|e| e.to_string())?;

    println!(
        "Added level {} from source level {}",
        args.target_level, source_level
    );

    Ok(())
}
