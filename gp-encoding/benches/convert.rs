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

    for file_path in files {
        let file_name = file_path.file_name().unwrap().to_string_lossy().into_owned();
        let file_size = file_path.metadata().map(|m| m.len()).unwrap_or(0);

        // Skip massive files (> 10 MB) to keep benchmarks running within reasonable time
        if file_size > 10 * 1024 * 1024 {
            continue;
        }

        let output_store = output_base.join(format!("bench_{}", file_path.file_stem().unwrap().to_string_lossy()));

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
}

criterion_group!(benches, bench_convert_geotiffs);
criterion_main!(benches);
