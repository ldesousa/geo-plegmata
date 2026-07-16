// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::{Path, PathBuf};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main, black_box};
use geoplegma::types::{DggrsUid, Point, RefinementLevel};
use gp_encoding::{ZarrBackend, convert_geotiff_file_to_backend, StorageBackend};
use gp_encoding::query::query_value_for_point;
use gp_encoding::value::decode_value_to_f64;
use gdal::{Dataset, GeoTransformEx};
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use rand::Rng;

// benchmark configuration
const DGGRS_TYPES: &[DggrsUid] = &[DggrsUid::H3];
const BANDS: &[i32] = &[1];


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

fn bench_convert_geotiffs_impl(c: &mut Criterion, dggrs_type: DggrsUid) {
    // Hardcoded folder with input GeoTIFF files
    let folder = Path::new("benches/files");
    let output_base = Path::new("benches/bench_out");

    let files = find_tiff_files(folder).unwrap_or_default();
    if files.is_empty() {
        eprintln!("Warning: No .tif/.tiff files found in hardcoded directory '{}'", folder.display());
        return;
    }

    let group_name = format!("convert_geotiff_{:?}", dggrs_type);
    let mut group = c.benchmark_group(&group_name);
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

        let output_store = output_base.join(format!("bench_{:?}_{}", dggrs_type, file_path.file_stem().unwrap().to_string_lossy()));

        // Run once to measure size
        if output_store.exists() {
            let _ = std::fs::remove_dir_all(&output_store);
        }
        let res = convert_geotiff_file_to_backend::<ZarrBackend>(
            &file_path,
            &output_store,
            dggrs_type,
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
            b.iter_batched(
                || {
                    if output_store.exists() {
                        let _ = std::fs::remove_dir_all(&output_store);
                    }
                },
                |_| {
                    let res = convert_geotiff_file_to_backend::<ZarrBackend>(
                        black_box(&file_path),
                        black_box(&output_store),
                        black_box(dggrs_type),
                        black_box(None),
                    );

                    assert!(res.is_ok(), "Conversion failed: {:?}", res.err());
                },
                BatchSize::SmallInput,
            )
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
        println!("│              GeoTIFF to Zarr Size Report ({:?})                           │", dggrs_type);
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
            let report_path = output_base.join(format!("size_report_{:?}.md", dggrs_type));
            let mut md_content = String::new();
            md_content.push_str(&format!("# GeoTIFF to Zarr Size Comparison Report ({:?})\n\n", dggrs_type));
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

fn bench_convert_geotiffs(c: &mut Criterion) {
    for &dggrs_type in DGGRS_TYPES {
        bench_convert_geotiffs_impl(c, dggrs_type);
    }
}

struct AccuracyComparison {
    file_name: String,
    samples_count: usize,
    matches_count: usize,
    mismatches_count: usize,
}

fn bench_query_accuracy_impl(c: &mut Criterion, dggrs_type: DggrsUid, band_num: i32) {
    let folder = Path::new("benches/files");
    let output_base = Path::new("benches/bench_out");

    let files = find_tiff_files(folder).unwrap_or_default();
    if files.is_empty() {
        eprintln!("Warning: No .tif/.tiff files found in hardcoded directory '{}'", folder.display());
        return;
    }

    let group_name = format!("query_accuracy_{:?}_band{}", dggrs_type, band_num);
    let mut group = c.benchmark_group(&group_name);
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(5));

    let mut accuracy_reports = Vec::new();

    for file_path in files {
        let file_name = file_path.file_name().unwrap().to_string_lossy().into_owned();
        let file_size = file_path.metadata().map(|m| m.len()).unwrap_or(0);

        if file_size > 10 * 1024 * 1024 {
            continue;
        }

        let output_store = output_base.join(format!("accuracy_{:?}_band{}_{}", dggrs_type, band_num, file_path.file_stem().unwrap().to_string_lossy()));

        if output_store.exists() {
            let _ = std::fs::remove_dir_all(&output_store);
        }

        let conversion_res = convert_geotiff_file_to_backend::<ZarrBackend>(
            &file_path,
            &output_store,
            dggrs_type,
            None,
        );

        let (backend, _, report) = match conversion_res {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Warning: Failed to convert {:?} for accuracy benchmark: {:?}", file_path, e);
                continue;
            }
        };

        let dataset = match Dataset::open(&file_path) {
            Ok(ds) => ds,
            Err(e) => {
                eprintln!("Warning: Failed to open original GeoTIFF {:?}: {:?}", file_path, e);
                let _ = std::fs::remove_dir_all(&output_store);
                continue;
            }
        };

        let (width, height) = dataset.raster_size();
        let gt = match dataset.geo_transform() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Warning: Failed to get geotransform for {:?}: {:?}", file_path, e);
                let _ = std::fs::remove_dir_all(&output_store);
                continue;
            }
        };

        let src_srs = match dataset.spatial_ref() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: Failed to get spatial ref for {:?}: {:?}", file_path, e);
                let _ = std::fs::remove_dir_all(&output_store);
                continue;
            }
        };

        let mut wgs84 = match SpatialRef::from_epsg(4326) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Warning: Failed to create WGS84 SRS: {:?}", e);
                let _ = std::fs::remove_dir_all(&output_store);
                continue;
            }
        };
        wgs84.set_axis_mapping_strategy(gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder);

        let to_wgs84 = match CoordTransform::new(&src_srs, &wgs84) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Warning: Failed to create CoordTransform: {:?}", e);
                let _ = std::fs::remove_dir_all(&output_store);
                continue;
            }
        };

        let band = match dataset.rasterband(band_num as usize) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Warning: Failed to get rasterband {}: {:?}", band_num, e);
                let _ = std::fs::remove_dir_all(&output_store);
                continue;
            }
        };

        let refinement_level = RefinementLevel::new(report.refinement_level as i32).unwrap();
        let dtype = &backend.metadata().attributes[0].dtype;

        let mut rng = rand::thread_rng();
        let sample_size = 1000;
        let mut samples = Vec::with_capacity(sample_size);

        for _ in 0..sample_size {
            let col = rng.gen_range(0..width);
            let row = rng.gen_range(0..height);

            let pixel_val = match band.read_as::<f64>((col as isize, row as isize), (1, 1), (1, 1), None) {
                Ok(buf) => buf.data()[0],
                Err(_) => f64::NAN,
            };

            let dx = rng.gen_range(-0.5..0.5);
            let dy = rng.gen_range(-0.5..0.5);
            let (x_geo, y_geo) = gt.apply(col as f64 + 0.5 + dx, row as f64 + 0.5 + dy);
            let mut xs = vec![x_geo];
            let mut ys = vec![y_geo];
            let mut zs = vec![];
            
            let point = if to_wgs84.transform_coords(&mut xs, &mut ys, &mut zs).is_ok() {
                Some(Point::new(ys[0], xs[0]))
            } else {
                None
            };

            if let Some(pt) = point {
                samples.push((pt, pixel_val));
            }
        }

        let mut matches_count = 0;
        let mut mismatches_count = 0;

        for (pt, original_val) in &samples {
            let query_res = query_value_for_point(&backend, refinement_level, 0, *pt);
            let encoded_val = match query_res {
                Ok(bytes) => decode_value_to_f64(dtype, &bytes).unwrap_or(f64::NAN),
                Err(_) => f64::NAN,
            };

            let is_match = if original_val.is_nan() && encoded_val.is_nan() {
                true
            } else if original_val.is_nan() || encoded_val.is_nan() {
                false
            } else {
                original_val == &encoded_val
            };

            if is_match {
                matches_count += 1;
            } else {
                mismatches_count += 1;
            }
        }

        accuracy_reports.push(AccuracyComparison {
            file_name: file_name.clone(),
            samples_count: samples.len(),
            matches_count,
            mismatches_count,
        });

        group.bench_function(format!("query_{}", file_name), |b| {
            let mut idx = 0;
            b.iter(|| {
                let (pt, _) = &samples[idx % samples.len()];
                idx += 1;
                let _ = query_value_for_point(black_box(&backend), black_box(refinement_level), black_box(0), black_box(*pt));
            })
        });

        if output_store.exists() {
            let _ = std::fs::remove_dir_all(&output_store);
        }
    }

    group.finish();

    if !accuracy_reports.is_empty() {
        println!("\n┌────────────────────────────────────────────────────────────────────────────────────────┐");
        println!("│      gp-encoding Query Accuracy Report ({:?}, Band {})                          │", dggrs_type, band_num);
        println!("├──────────────────────┬───────────────┬───────────────┬───────────────┬─────────────────┤");
        println!("│ {:<20} │ {:<13} │ {:<13} │ {:<13} │ {:<15} │", "File Name", "Total Samples", "Matches", "Mismatches", "Match Rate");
        println!("├──────────────────────┼───────────────┼───────────────┼───────────────┼─────────────────┤");
        for report in &accuracy_reports {
            let match_rate = if report.samples_count == 0 {
                "0.0%".to_string()
            } else {
                format!("{:.2}%", (report.matches_count as f64 / report.samples_count as f64) * 100.0)
            };
            println!(
                "│ {:<20} │ {:>13} │ {:>13} │ {:>13} │ {:>15} │",
                report.file_name,
                report.samples_count,
                report.matches_count,
                report.mismatches_count,
                match_rate
            );
        }
        println!("└──────────────────────┴───────────────┴───────────────┴───────────────┴─────────────────┘\n");

        if let Err(e) = std::fs::create_dir_all(output_base) {
            eprintln!("Warning: Failed to create output directory for accuracy report: {:?}", e);
        } else {
            let report_path = output_base.join(format!("accuracy_report_{:?}_band{}.md", dggrs_type, band_num));
            let mut md_content = String::new();
            md_content.push_str(&format!("# gp-encoding Query Accuracy Report ({:?}, Band {})\n\n", dggrs_type, band_num));
            md_content.push_str("| File Name | Total Samples | Matches | Mismatches | Match Rate |\n");
            md_content.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
            for report in &accuracy_reports {
                let match_rate = if report.samples_count == 0 {
                    "0.0%".to_string()
                } else {
                    format!("{:.2}%", (report.matches_count as f64 / report.samples_count as f64) * 100.0)
                };
                md_content.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    report.file_name,
                    report.samples_count,
                    report.matches_count,
                    report.mismatches_count,
                    match_rate
                ));
            }
            if let Err(e) = std::fs::write(&report_path, md_content) {
                eprintln!("Warning: Failed to write accuracy report to {:?}: {:?}", report_path, e);
            } else {
                println!("Accuracy report successfully written to {:?}", report_path);
            }
        }
    }
}

fn bench_query_accuracy(c: &mut Criterion) {
    for &dggrs_type in DGGRS_TYPES {
        for &band_num in BANDS {
            bench_query_accuracy_impl(c, dggrs_type, band_num);
        }
    }
}

criterion_group!(benches, bench_convert_geotiffs, bench_query_accuracy);
criterion_main!(benches);
