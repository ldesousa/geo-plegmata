// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::{Path, PathBuf};
use criterion::{Criterion, criterion_group, criterion_main, black_box};
use geoplegma::types::DggrsUid;
use gp_encoding::{ZarrBackend, convert_geotiff_file_to_backend};

fn find_tiff_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    find_tiff_files_rec(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn find_tiff_files_rec(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == "target" || name.starts_with('.') {
                    continue;
                }
            }
            find_tiff_files_rec(&path, files)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "tif" || ext_lower == "tiff" {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

fn get_dir_size(dir: &Path) -> std::io::Result<u64> {
    if !dir.exists() {
        return Ok(0);
    }
    let mut total_size = 0;
    if dir.is_file() {
        total_size += dir.metadata()?.len();
    } else if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total_size += get_dir_size(&path)?;
            } else {
                total_size += path.metadata()?.len();
            }
        }
    }
    Ok(total_size)
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_ratio(out_size: u64, in_size: u64) -> String {
    if in_size == 0 {
        "N/A".to_string()
    } else {
        format!("{:.1}%", (out_size as f64 / in_size as f64) * 100.0)
    }
}

struct SizeComparison {
    file_name: String,
    input_size: u64,
    output_size: u64,
}

fn bench_convert_geotiffs(c: &mut Criterion) {
    // Hardcoded folder with input GeoTIFF files
    let folder = Path::new("benches/files");
    let output_base = Path::new("benches/bench_out");

    let files = find_tiff_files(folder).unwrap_or_default();
    if files.is_empty() {
        eprintln!("Warning: No .tif/.tiff files found in hardcoded directory '{}'", folder.display());
        return;
    }

    let mut group = c.benchmark_group("convert_geotiff");
    // Limit sample size and measurement time because file conversion is slow
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    let mut comparisons = Vec::new();

    for file_path in files {
        let file_name = file_path.file_name().unwrap().to_string_lossy().into_owned();
        let file_size = file_path.metadata().map(|m| m.len()).unwrap_or(0);

        // Skip massive files (> 10 MB) to keep benchmarks running within reasonable time
        if file_size > 10 * 1024 * 1024 {
            continue;
        }

        let output_store = output_base.join(format!("bench_{}", file_path.file_stem().unwrap().to_string_lossy()));

        // Run once to measure size
        if output_store.exists() {
            let _ = std::fs::remove_dir_all(&output_store);
        }
        let res = convert_geotiff_file_to_backend::<ZarrBackend>(
            &file_path,
            &output_store,
            DggrsUid::H3,
            None,
        );
        if res.is_ok() {
            let out_size = get_dir_size(&output_store).unwrap_or(0);
            comparisons.push(SizeComparison {
                file_name: file_name.clone(),
                input_size: file_size,
                output_size: out_size,
            });
        }

        group.bench_function(&file_name, |b| {
            b.iter(|| {
                // Ensure output directory is clean before the run
                if output_store.exists() {
                    let _ = std::fs::remove_dir_all(&output_store);
                }

                let res = convert_geotiff_file_to_backend::<ZarrBackend>(
                    black_box(&file_path),
                    black_box(&output_store),
                    black_box(DggrsUid::H3),
                    black_box(None),
                );

                assert!(res.is_ok(), "Conversion failed: {:?}", res.err());
            })
        });

        // Clean up output directories after benchmarking this file
        if output_store.exists() {
            let _ = std::fs::remove_dir_all(&output_store);
        }
    }

    group.finish();

    // Print and write size report
    if !comparisons.is_empty() {
        println!("\n┌────────────────────────────────────────────────────────────────────────────────────────┐");
        println!("│                              GeoTIFF to Zarr Size Report                               │");
        println!("├──────────────────────┬──────────────────┬──────────────────┬───────────────────────────┤");
        println!("│ {:<20} │ {:<16} │ {:<16} │ {:<25} │", "File Name", "Input Size", "Output Size", "Ratio (Out/In)");
        println!("├──────────────────────┼──────────────────┼──────────────────┼───────────────────────────┤");
        for comp in &comparisons {
            println!(
                "│ {:<20} │ {:>16} │ {:>16} │ {:>25} │",
                comp.file_name,
                format_size(comp.input_size),
                format_size(comp.output_size),
                format_ratio(comp.output_size, comp.input_size)
            );
        }
        println!("└──────────────────────┴──────────────────┴──────────────────┴───────────────────────────┘\n");

        // Write to Markdown file
        if let Err(e) = std::fs::create_dir_all(output_base) {
            eprintln!("Warning: Failed to create output directory for report: {:?}", e);
        } else {
            let report_path = output_base.join("size_report.md");
            let mut md_content = String::new();
            md_content.push_str("# GeoTIFF to Zarr Size Comparison Report\n\n");
            md_content.push_str("| File Name | Input Size | Output Size | Ratio (Out/In) |\n");
            md_content.push_str("| :--- | :--- | :--- | :--- |\n");
            for comp in &comparisons {
                md_content.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    comp.file_name,
                    format_size(comp.input_size),
                    format_size(comp.output_size),
                    format_ratio(comp.output_size, comp.input_size)
                ));
            }
            if let Err(e) = std::fs::write(&report_path, md_content) {
                eprintln!("Warning: Failed to write size report to {:?}: {:?}", report_path, e);
            } else {
                println!("Size report successfully written to {:?}", report_path);
            }
        }
    }
}

criterion_group!(benches, bench_convert_geotiffs);
criterion_main!(benches);
