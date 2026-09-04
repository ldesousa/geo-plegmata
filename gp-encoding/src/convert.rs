// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;

use gdal::raster::{GdalDataType, RasterBand};
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::{Dataset, GeoTransformEx, Metadata};
use geoplegma::api::DggrsApiConfig;
use geoplegma::get;
use geoplegma::types::{BoundingBox, DggrsUid, Point, RefinementLevel, RelativeDepth, ZoneId};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::AttributeSchema;
use crate::common::{CONFIG, ID_ONLY_CONFIG};
use crate::error::EncodingError;
use crate::models::{Compression, DataType, DatasetMetadata};
use crate::stats::{BandStatsCollector, ConversionReport, SourceRasterReport};
use crate::storage::StorageBackend;
use crate::value::{encode_value_from_f64, parse_fill_value_to_f64};

trait NativeBytes {
    fn to_native_bytes(self) -> Vec<u8>;
    fn to_f64(self) -> f64;
}

macro_rules! impl_native_bytes {
    ($($t:ty),+ $(,)?) => {
        $(
            impl NativeBytes for $t {
                fn to_native_bytes(self) -> Vec<u8> {
                    self.to_ne_bytes().to_vec()
                }
                fn to_f64(self) -> f64 {
                    self as f64
                }
            }
        )+
    };
}

impl_native_bytes!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

const ZARR_TARGET_UNCOMPRESSED_CHUNK_BYTES: u64 = 1024 * 1024;

fn attribute_schema_from_band(band: &RasterBand<'_>) -> Result<AttributeSchema, EncodingError> {
    let band_type = band.band_type();
    let dtype = match band_type {
        GdalDataType::UInt8 => DataType::UInt8,
        GdalDataType::Int8 => DataType::Int8,
        GdalDataType::Int16 => DataType::Int16,
        GdalDataType::UInt16 => DataType::UInt16,
        GdalDataType::Int32 => DataType::Int32,
        GdalDataType::UInt32 => DataType::UInt32,
        GdalDataType::Int64 => DataType::Int64,
        GdalDataType::UInt64 => DataType::UInt64,
        GdalDataType::Float32 => DataType::Float32,
        GdalDataType::Float64 => DataType::Float64,
        _ => {
            return Err(EncodingError::Dataset(format!(
                "unsupported GDAL data type: {band_type:?}"
            )));
        }
    };

    Ok(AttributeSchema {
        dtype,
        fill_value: band
            .no_data_value()
            .map(|value| value.to_string())
            .unwrap_or_else(|| dtype.default_fill_value()),
    })
}

fn get_corners_and_pixel_size(
    dataset: &Dataset,
) -> Result<(Option<BoundingBox>, f64, f64), EncodingError> {
    let (width_px, height_px) = dataset.raster_size();
    let w = width_px as f64;
    let h = height_px as f64;

    let gt = dataset.geo_transform()?;
    let src_srs = dataset.spatial_ref().unwrap_or_else(|_| {
        let mut srs = SpatialRef::from_epsg(4326).unwrap();
        srs.set_axis_mapping_strategy(gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder);
        srs
    });

    let mut wgs84 = SpatialRef::from_epsg(4326)?;
    wgs84.set_axis_mapping_strategy(gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder);
    let to_wgs84 = CoordTransform::new(&src_srs, &wgs84)?;

    let (ulx, uly) = gt.apply(0.0, 0.0);
    let (urx, ury) = gt.apply(w, 0.0);
    let (lrx, lry) = gt.apply(w, h);
    let (llx, lly) = gt.apply(0.0, h);

    let mut xs = vec![ulx, urx, lrx, llx];
    let mut ys = vec![uly, ury, lry, lly];
    let mut zs = vec![];
    to_wgs84.transform_coords(&mut xs, &mut ys, &mut zs)?;

    let lon_min = xs.iter().fold(f64::INFINITY, |acc, x| acc.min(*x));
    let lon_max = xs.iter().fold(f64::NEG_INFINITY, |acc, x| acc.max(*x));
    let lat_min = ys.iter().fold(f64::INFINITY, |acc, y| acc.min(*y));
    let lat_max = ys.iter().fold(f64::NEG_INFINITY, |acc, y| acc.max(*y));

    println!("Bounding box (WGS84):");
    println!("  lon: [{lon_min:.6}, {lon_max:.6}]");
    println!("  lat: [{lat_min:.6}, {lat_max:.6}]");

    let cx = gt[0] + (w / 2.0) * gt[1] + (h / 2.0) * gt[2];
    let cy = gt[3] + (w / 2.0) * gt[4] + (h / 2.0) * gt[5];

    let metric = SpatialRef::from_epsg(3857)?;
    let to_metric = CoordTransform::new(&src_srs, &metric)?;

    let mut px = vec![cx, cx + gt[1], cx + gt[2]];
    let mut py = vec![cy, cy + gt[4], cy + gt[5]];
    let mut pz = vec![];
    to_metric.transform_coords(&mut px, &mut py, &mut pz)?;

    let pixel_w = f64::hypot(px[1] - px[0], py[1] - py[0]);
    let pixel_h = f64::hypot(px[2] - px[0], py[2] - py[0]);

    let tolerance = 1e-4; // 0.0001 degrees
    let is_global = (lon_min <= -180.0 + tolerance)
        && (lon_max >= 180.0 - tolerance)
        && (lat_min <= -90.0 + tolerance)
        && (lat_max >= 90.0 - tolerance);

    let bbox = if is_global {
        None
    } else {
        Some(BoundingBox::new(lon_min, lat_min, lon_max, lat_max))
    };

    Ok((bbox, pixel_w, pixel_h))
}

fn nearest_pixel_coord_for_center(
    center: &Point,
    wgs84_to_src: &CoordTransform,
    gt: [f64; 6],
    width: usize,
    height: usize,
) -> Result<Option<(usize, usize)>, EncodingError> {
    if width == 0 || height == 0 {
        return Err(EncodingError::Dataset(
            "raster has zero width or height".into(),
        ));
    }

    let mut xs = vec![center.lon];
    let mut ys = vec![center.lat];
    let mut zs = vec![];
    wgs84_to_src.transform_coords(&mut xs, &mut ys, &mut zs)?;

    let det = gt[1] * gt[5] - gt[2] * gt[4];
    if det.abs() < f64::EPSILON {
        return Err(EncodingError::Dataset(
            "geotransform is not invertible".into(),
        ));
    }

    let dx = xs[0] - gt[0];
    let dy = ys[0] - gt[3];

    // Inverse affine transform gives corner-based pixel coordinates.
    // Subtract 0.5 so rounding selects the nearest pixel center.
    let col_corner = (gt[5] * dx - gt[2] * dy) / det;
    let row_corner = (-gt[4] * dx + gt[1] * dy) / det;

    let col = (col_corner - 0.5).round();
    let row = (row_corner - 0.5).round();

    // Do not clamp. If center is outside raster extent, keep fill value.
    if !(0.0..(width as f64)).contains(&col) || !(0.0..(height as f64)).contains(&row) {
        return Ok(None);
    }

    Ok(Some((col as usize, row as usize)))
}

fn get_closest_refinement_level(
    grid: &std::sync::Arc<dyn geoplegma::api::DggrsApi>,
    pixel_width: f64,
    pixel_height: f64,
) -> Result<RefinementLevel, EncodingError> {
    if pixel_width == 0.0 || pixel_height == 0.0 {
        return Err(EncodingError::Dataset(
            "geotransform has zero pixel size".into(),
        ));
    }

    let world_pixel_count =
        ((40_075_016.685 / pixel_width) * (40_075_016.685 / pixel_height)) as u64; // TODO: support different projections?

    println!("world pixel count: {}", world_pixel_count);
    let mut best_level: Option<RefinementLevel> = None;
    let mut best_diff = f64::MAX;

    let min_level = grid.min_refinement_level()?;
    let max_level = grid.max_refinement_level()?;

    for raw_level in min_level.get()..=max_level.get() {
        let level = RefinementLevel::new_const(raw_level);

        let zone_count = grid.zone_count(level)?;

        let ratio = if world_pixel_count > zone_count {
            world_pixel_count as f64 / zone_count as f64
        } else {
            zone_count as f64 / world_pixel_count as f64
        };

        if ratio < best_diff {
            best_diff = ratio;
            best_level = Some(level);
        }
    }
    let diff_percentage = (best_diff - 1.0) * 100.0;
    println!(
        "best level: {} with diff {}%",
        best_level.unwrap().get(),
        diff_percentage
    );

    best_level.ok_or_else(|| EncodingError::Grid("no valid refinement level found".into()))
}

pub fn choose_best_chunk_level_and_size(
    refinement_level: RefinementLevel,
    min_chunk_level: RefinementLevel,
    max_relative_depth_allowed: RelativeDepth,
    aperture: u64,
    data_type_size_bytes: usize,
) -> Result<(RefinementLevel, u64), EncodingError> {
    if data_type_size_bytes == 0 {
        return Err(EncodingError::Storage(
            "data type size must be greater than zero".into(),
        ));
    }
    if aperture < 2 {
        return Err(EncodingError::Grid(
            "grid aperture must be at least 2 for chunk sizing".into(),
        ));
    }
    if min_chunk_level.get() > refinement_level.get() {
        return Err(EncodingError::Storage(format!(
            "min chunk level ({}) is greater than refinement level ({})",
            min_chunk_level.get(),
            refinement_level.get()
        )));
    }

    let max_relative_depth_from_levels = refinement_level.get() - min_chunk_level.get();
    let max_relative_depth = max_relative_depth_from_levels.min(max_relative_depth_allowed.get());
    let target_bytes = ZARR_TARGET_UNCOMPRESSED_CHUNK_BYTES as u128;
    let dtype_bytes = data_type_size_bytes as u128;

    let mut best_relative_depth = 0i32;
    let mut best_chunk_cells = 1u128;
    let mut best_diff = target_bytes.abs_diff(dtype_bytes); // depth 0 => 1 cell

    let mut chunk_cells = 1u128;
    for relative_depth in 1..=max_relative_depth {
        chunk_cells = chunk_cells.checked_mul(aperture as u128).ok_or_else(|| {
            EncodingError::Storage("chunk size overflow while computing aperture growth".into())
        })?;

        let chunk_bytes = chunk_cells.checked_mul(dtype_bytes).ok_or_else(|| {
            EncodingError::Storage("chunk byte size overflow while tuning chunk level".into())
        })?;

        let diff = target_bytes.abs_diff(chunk_bytes);
        if diff < best_diff {
            best_diff = diff;
            best_relative_depth = relative_depth;
            best_chunk_cells = chunk_cells;
        }
    }

    let chunk_level = RefinementLevel::new_const(refinement_level.get() - best_relative_depth);
    let chunk_size = u64::try_from(best_chunk_cells)
        .map_err(|_| EncodingError::Storage("chunk size does not fit into u64".into()))?;

    let chunk_size = (chunk_size as f64 * 1.05) as u64;

    Ok((chunk_level, chunk_size))
}

pub fn compute_source_report(dataset: &Dataset) -> Result<SourceRasterReport, EncodingError> {
    let (width, height) = dataset.raster_size();
    let total_pixels = (width as u64) * (height as u64);
    let band_count = dataset.raster_count();
    let mut bands = Vec::with_capacity(band_count);

    for band_idx in 1..=band_count {
        let band = dataset.rasterband(band_idx)?;
        let dtype_name = format!("{:?}", band.band_type().name());
        let nodata = band.no_data_value();

        let mut collector = BandStatsCollector::new((band_idx - 1) as u32, dtype_name);
        collector.set_total_cells(total_pixels);

        let chunk_w = 4096;
        let chunk_h = 4096;
        for y in (0..height).step_by(chunk_h) {
            let h = chunk_h.min(height - y);
            for x in (0..width).step_by(chunk_w) {
                let w = chunk_w.min(width - x);
                let raster = band.read_as::<f64>(
                    (x as isize, y as isize),
                    (w, h),
                    (w, h),
                    None,
                )?;
                for &v in raster.data() {
                    let is_nodata = match nodata {
                        Some(nd) => v == nd || (v.is_nan() && nd.is_nan()),
                        None => false,
                    };
                    if !is_nodata && v.is_finite() {
                        collector.record_value(v);
                    }
                }
            }
        }

        bands.push(collector.finish());
    }

    Ok(SourceRasterReport {
        width,
        height,
        total_pixels,
        bands,
    })
}

pub fn list_subdatasets(dataset: &Dataset) -> Vec<(String, String)> {
    let mut subdatasets = Vec::new();
    if let Some(domain) = dataset.metadata_domain("SUBDATASETS") {
        let mut name_map = std::collections::BTreeMap::new();
        let mut desc_map = std::collections::BTreeMap::new();
        for item in domain {
            if let Some((key, value)) = item.split_once('=') {
                if key.starts_with("SUBDATASET_") {
                    if key.ends_with("_NAME") {
                        if let Some(num_str) = key.strip_prefix("SUBDATASET_").and_then(|k| k.strip_suffix("_NAME")) {
                            if let Ok(num) = num_str.parse::<usize>() {
                                name_map.insert(num, value.to_string());
                            }
                        }
                    } else if key.ends_with("_DESC") {
                        if let Some(num_str) = key.strip_prefix("SUBDATASET_").and_then(|k| k.strip_suffix("_DESC")) {
                            if let Ok(num) = num_str.parse::<usize>() {
                                desc_map.insert(num, value.to_string());
                            }
                        }
                    }
                }
            }
        }
        for (num, name) in name_map {
            let desc = desc_map.get(&num).cloned().unwrap_or_default();
            subdatasets.push((name, desc));
        }
    }
    subdatasets
}

pub fn get_subdataset_short_name(name: &str) -> String {
    let last_part = name.split(':').last().unwrap_or(name);
    last_part.trim_matches(|c| c == '/' || c == '"' || c == '\'' || c == ' ').to_string()
}

pub fn convert_to_backend<B>(
    input_str: &str,
    subdataset: Option<&str>,
    output_path: &Path,
    dggrs: DggrsUid,
    compression: Option<Compression>,
) -> Result<(B, SourceRasterReport, ConversionReport), EncodingError>
where
    B: StorageBackend,
{
    let grid = get(dggrs)?;

    let initial_dataset = Dataset::open(input_str).map_err(|e| {
        EncodingError::Dataset(format!(
            "failed to open input dataset '{}': {e}",
            input_str
        ))
    })?;

    let subdatasets = list_subdatasets(&initial_dataset);
    let (dataset, open_str) = if !subdatasets.is_empty() {
        if let Some(sub) = subdataset {
            let matched = subdatasets.iter().find(|(name, _)| {
                let short_name = get_subdataset_short_name(name);
                short_name.eq_ignore_ascii_case(sub) || short_name.to_lowercase().contains(&sub.to_lowercase())
            });
            if let Some((name, _)) = matched {
                let ds = Dataset::open(name).map_err(|e| {
                    EncodingError::Dataset(format!(
                        "failed to open subdataset '{}': {e}",
                        name
                    ))
                })?;
                (ds, name.clone())
            } else {
                let mut msg = format!(
                    "Subdataset '{}' not found in input. Available subdatasets:\n",
                    sub
                );
                for (name, desc) in &subdatasets {
                    let short_name = get_subdataset_short_name(name);
                    msg.push_str(&format!("  - {} ({})\n", short_name, desc));
                }
                return Err(EncodingError::Dataset(msg));
            }
        } else {
            let mut msg = format!(
                "Input dataset '{}' contains multiple subdatasets. Please specify one using --subdataset <name>.\nAvailable subdatasets:\n",
                input_str
            );
            for (name, desc) in &subdatasets {
                let short_name = get_subdataset_short_name(name);
                msg.push_str(&format!("  - {} ({})\n", short_name, desc));
            }
            return Err(EncodingError::Dataset(msg));
        }
    } else {
        if let Some(sub) = subdataset {
            return Err(EncodingError::Dataset(format!(
                "Input dataset does not contain subdatasets, but --subdataset '{}' was specified.",
                sub
            )));
        }
        (initial_dataset, input_str.to_string())
    };

    let source_report = compute_source_report(&dataset)?;

    let bands = dataset
        .rasterbands()
        .map(|b| b.map(|band| band.band_type()))
        .collect::<Result<Vec<_>, _>>()?;
    let metadata_bands = dataset
        .rasterbands()
        .map(|band| attribute_schema_from_band(&band?))
        .collect::<Result<Vec<_>, EncodingError>>()?;

    if metadata_bands.is_empty() {
        return Err(EncodingError::Storage(
            "dataset metadata must define at least one attribute".into(),
        ));
    }

    let (width, height) = dataset.raster_size();

    if width == 0 || height == 0 {
        return Err(EncodingError::Dataset(
            "raster has zero width or height".into(),
        ));
    }

    let gt = dataset.geo_transform()?;
    let src_srs = dataset.spatial_ref().unwrap_or_else(|_| {
        let mut srs = SpatialRef::from_epsg(4326).unwrap();
        srs.set_axis_mapping_strategy(gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder);
        srs
    });
    let (bbox, pixel_width, pixel_height) = get_corners_and_pixel_size(&dataset)?;
    let refinement_level = get_closest_refinement_level(&grid, pixel_width, pixel_height)?;

    let data_type_size_bytes = metadata_bands
        .iter()
        .map(|band| band.dtype.byte_size())
        .max()
        .ok_or_else(|| {
            EncodingError::Storage("dataset metadata must define at least one attribute".into())
        })?;
    let min_chunk_level = grid.min_refinement_level()?;
    let max_relative_depth_allowed = grid.max_relative_depth()?;
    let (chunk_level, chunk_size) = choose_best_chunk_level_and_size(
        refinement_level,
        min_chunk_level,
        max_relative_depth_allowed,
        dggrs.spec().aperture as u64,
        data_type_size_bytes,
    )?;
    println!(
        "refinement level: {}, chunk level: {}, chunk size: {}, chunk bytes: {}",
        refinement_level.get(),
        chunk_level.get(),
        chunk_size,
        chunk_size * data_type_size_bytes as u64
    );

    let chunk_zones = grid.zones_from_bbox(chunk_level, bbox, Some(CONFIG))?;
    if chunk_zones.zones.is_empty() {
        return Err(EncodingError::Grid(
            "no zones found intersecting dataset bounding box".into(),
        ));
    }
    println!("zones in bbox: {}", chunk_zones.zones.len());

    let relative_depth = RelativeDepth::new(refinement_level.get() - chunk_level.get())?;
    let center_config = DggrsApiConfig {
        center: true,
        ..CONFIG
    };
    let total_chunk_zones = chunk_zones.zones.len();
    let chunk_progress = ProgressBar::new(total_chunk_zones as u64);
    let style = ProgressStyle::with_template(
        "processing chunk zones [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)",
    )
    .map_err(|e| EncodingError::Storage(format!("invalid progress bar template: {e}")))?
    .progress_chars("=> ");
    chunk_progress.set_style(style);

    let band_dtype_names: Vec<String> = metadata_bands
        .iter()
        .map(|b| format!("{:?}", b.dtype))
        .collect();

    let chunk_ids = chunk_zones
        .zones
        .iter()
        .map(|z| z.id.to_string())
        .collect::<Vec<_>>();

    let metadata = DatasetMetadata {
        dggrs,
        attributes: metadata_bands,
        chunk_size,
        levels: vec![refinement_level.get() as u32],
        compression,
    };

    let encoded_num_cells = (chunk_zones.zones.len() as u64)
        .checked_mul(chunk_size)
        .ok_or_else(|| EncodingError::Storage("encoded cell count overflow".into()))?;
    let mut backend = B::create(output_path, metadata)?;
    backend.set_level_chunk_ids(refinement_level.get() as u32, chunk_level.get() as u32, chunk_ids.clone())?;

    let src_srs_wkt = src_srs.to_wkt()?;
    let band_count = bands.len();

    for band_idx in 0..band_count {
        backend.create_level(
            refinement_level.get() as u32,
            band_idx as u32,
            encoded_num_cells,
            chunk_size,
        )?;
    }

    let fill_value_bytes: Vec<Vec<u8>> = backend
        .metadata()
        .attributes
        .iter()
        .map(|attr| {
            let fill_val = parse_fill_value_to_f64(&attr.dtype, &attr.fill_value)?;
            encode_value_from_f64(&attr.dtype, fill_val)
        })
        .collect::<Result<_, EncodingError>>()?;

    let progress_counter = std::sync::atomic::AtomicU64::new(0);

    let open_str_clone = open_str.clone();

    let results: Vec<Vec<BandStatsCollector>> = chunk_zones
        .zones
        .par_iter()
        .enumerate()
        .map_init(
            || -> Result<(CoordTransform, Dataset), EncodingError> {
                let mut wgs84 = SpatialRef::from_epsg(4326)?;
                wgs84.set_axis_mapping_strategy(
                    gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder,
                );
                let mut src_srs = SpatialRef::from_wkt(&src_srs_wkt)?;
                src_srs.set_axis_mapping_strategy(
                    gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder,
                );
                let transform = CoordTransform::new(&wgs84, &src_srs)?;
                let thread_dataset = Dataset::open(&open_str_clone)?;
                Ok((transform, thread_dataset))
            },
            |state, (chunk_index, chunk_zone)| {
                let (wgs84_to_src, thread_dataset) = match state {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(EncodingError::Dataset(format!(
                            "failed to initialize thread-local transform/dataset: {e}"
                        )));
                    }
                };

                let mut local_collectors = Vec::with_capacity(band_count);
                for (band_idx, dtype_name) in band_dtype_names.iter().enumerate() {
                    local_collectors.push(BandStatsCollector::new(band_idx as u32, dtype_name.clone()));
                }

                let children = grid.zones_from_parent(relative_depth, chunk_zone.id.clone(), Some(center_config))?;
                if children.zones.len() > chunk_size as usize {
                    return Err(EncodingError::Grid(format!(
                        "chunk {} has {} children but chunk_size is {}",
                        chunk_zone.id,
                        children.zones.len(),
                        chunk_size
                    )));
                }

                let mut child_pixel_coords = Vec::with_capacity(children.zones.len());
                let mut min_col = usize::MAX;
                let mut max_col = 0;
                let mut min_row = usize::MAX;
                let mut max_row = 0;
                let mut has_valid_coords = false;

                for child in &children.zones {
                    let center = child.center.ok_or_else(|| {
                        EncodingError::Grid(format!(
                            "zone {} in chunk {} has no center coordinates",
                            child.id, chunk_zone.id
                        ))
                    })?;

                    let coord = nearest_pixel_coord_for_center(&center, wgs84_to_src, gt, width, height)?;
                    if let Some((col, row)) = coord {
                        min_col = min_col.min(col);
                        max_col = max_col.max(col);
                        min_row = min_row.min(row);
                        max_row = max_row.max(row);
                        has_valid_coords = true;
                    }
                    child_pixel_coords.push(coord);
                }

                for band_idx in 0..band_count {
                    let band = thread_dataset.rasterband(band_idx + 1)?;
                    let band_type = band.band_type();
                    let dtype = &backend.metadata().attributes[band_idx].dtype;
                    let val_size = dtype.byte_size();
                    let fill_val_bytes = &fill_value_bytes[band_idx];

                    let mut chunk_bytes = vec![0_u8; chunk_size as usize * val_size];
                    for chunk_cell_idx in 0..chunk_size as usize {
                        let start = chunk_cell_idx * val_size;
                        let end = start + val_size;
                        chunk_bytes[start..end].copy_from_slice(fill_val_bytes);
                    }

                    if has_valid_coords {
                        let window_w = max_col - min_col + 1;
                        let window_h = max_row - min_row + 1;
                        let window_area = window_w * window_h;

                        macro_rules! process_window {
                            ($t:ty) => {{
                                if window_area > 10_000_000 {
                                    for (in_chunk_idx, &coord_opt) in child_pixel_coords.iter().enumerate() {
                                        if let Some((col, row)) = coord_opt {
                                            let pixel_buf = band.read_as::<$t>(
                                                (col as isize, row as isize),
                                                (1, 1),
                                                (1, 1),
                                                None,
                                            )?;
                                            let val = pixel_buf.data()[0];
                                            local_collectors[band_idx].record_value(val.to_f64());
                                            let val_bytes = val.to_native_bytes();
                                            let start = in_chunk_idx * val_size;
                                            let end = start + val_size;
                                            chunk_bytes[start..end].copy_from_slice(&val_bytes);
                                        }
                                    }
                                } else {
                                    let buffer = band.read_as::<$t>(
                                        (min_col as isize, min_row as isize),
                                        (window_w, window_h),
                                        (window_w, window_h),
                                        None,
                                    )?;
                                    for (in_chunk_idx, &coord_opt) in child_pixel_coords.iter().enumerate() {
                                        if let Some((col, row)) = coord_opt {
                                            let local_col = col - min_col;
                                            let local_row = row - min_row;
                                            let val = buffer.data()[local_row * window_w + local_col];
                                            local_collectors[band_idx].record_value(val.to_f64());
                                            let val_bytes = val.to_native_bytes();
                                            let start = in_chunk_idx * val_size;
                                            let end = start + val_size;
                                            chunk_bytes[start..end].copy_from_slice(&val_bytes);
                                        }
                                    }
                                }
                            }};
                        }

                        match band_type {
                            GdalDataType::UInt8 => process_window!(u8),
                            GdalDataType::Int8 => process_window!(i8),
                            GdalDataType::UInt16 => process_window!(u16),
                            GdalDataType::Int16 => process_window!(i16),
                            GdalDataType::UInt32 => process_window!(u32),
                            GdalDataType::Int32 => process_window!(i32),
                            GdalDataType::UInt64 => process_window!(u64),
                            GdalDataType::Int64 => process_window!(i64),
                            GdalDataType::Float32 => process_window!(f32),
                            GdalDataType::Float64 => process_window!(f64),
                            _ => return Err(EncodingError::Dataset(format!(
                                "unsupported GDAL data type: {band_type:?}"
                            ))),
                        }
                    }

                    backend.write_chunk(
                        refinement_level.get() as u32,
                        band_idx as u32,
                        chunk_index as u64,
                        &chunk_bytes,
                    )?;
                }

                let done = progress_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                chunk_progress.set_position(done);

                Ok(local_collectors)
            }
        )
        .collect::<Result<Vec<_>, EncodingError>>()?;

    chunk_progress.finish_with_message("processing chunk zones [done]");

    let mut band_stats = Vec::with_capacity(band_count);
    for (band_idx, dtype_name) in band_dtype_names.iter().enumerate() {
        let mut main_collector = BandStatsCollector::new(band_idx as u32, dtype_name.clone());
        main_collector.set_total_cells(encoded_num_cells);
        for local_collectors in &results {
            main_collector.merge(&local_collectors[band_idx]);
        }
        band_stats.push(main_collector.finish());
    }

    let report = ConversionReport {
        num_chunks: chunk_zones.zones.len() as u64,
        chunk_size,
        chunk_level: chunk_level.get() as u32,
        refinement_level: refinement_level.get() as u32,
        bands: band_stats,
    };

    Ok((backend, source_report, report))
}

pub fn convert_dggrs_store_to_backend<B>(
    source_store_path: &Path,
    output_path: &Path,
    target_dggrs: DggrsUid,
    compression: Option<Compression>,
) -> Result<B, EncodingError>
where
    B: StorageBackend,
{
    let source_backend = B::open(source_store_path)?;
    let source_metadata = source_backend.metadata();
    let source_dggrs = source_metadata.dggrs;

    let source_grid = get(source_dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve source DGGS: {e}")))?;
    let target_grid = get(target_dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve target DGGS: {e}")))?;

    let source_levels = source_backend.levels();
    if source_levels.is_empty() {
        return Err(EncodingError::Storage("source store has no resolution levels".into()));
    }

    let mut level_mapping = Vec::new();
    let mut target_levels = Vec::new();

    for &src_lvl in &source_levels {
        let src_ref_lvl = RefinementLevel::new(src_lvl as i32)?;
        let src_count = source_grid.zone_count(src_ref_lvl)?;

        let mut best_level = target_grid.min_refinement_level()?;
        let mut best_diff = u64::MAX;
        for l in target_grid.min_refinement_level()?.get()..=target_grid.max_refinement_level()?.get() {
            let level = RefinementLevel::new(l)?;
            let count = target_grid.zone_count(level)?;
            let diff = src_count.abs_diff(count);
            if diff < best_diff {
                best_diff = diff;
                best_level = level;
            }
        }
        let tgt_lvl = best_level.get() as u32;
        level_mapping.push((src_lvl, tgt_lvl));
        target_levels.push(tgt_lvl);
    }

    target_levels.sort_unstable();
    target_levels.dedup();

    println!("Mapping levels from {} to {}:", source_dggrs, target_dggrs);
    for &(src, tgt) in &level_mapping {
        println!("  Source Level {} -> Target Level {}", src, tgt);
    }

    let max_target_level_u32 = target_levels.iter().max().copied().unwrap_or(0);
    let max_target_level = RefinementLevel::new(max_target_level_u32 as i32)?;

    let data_type_size_bytes = source_metadata.attributes
        .iter()
        .map(|band| band.dtype.byte_size())
        .max()
        .ok_or_else(|| {
            EncodingError::Storage("dataset metadata must define at least one attribute".into())
        })?;

    let target_min_chunk_level = target_grid.min_refinement_level()?;
    let target_max_relative_depth_allowed = target_grid.max_relative_depth()?;

    let (_best_chunk_level, chunk_size) = choose_best_chunk_level_and_size(
        max_target_level,
        target_min_chunk_level,
        target_max_relative_depth_allowed,
        target_dggrs.spec().aperture as u64,
        data_type_size_bytes,
    )?;

    let target_compression = compression.or_else(|| source_metadata.compression.clone());

    let target_metadata = DatasetMetadata {
        dggrs: target_dggrs,
        attributes: source_metadata.attributes.clone(),
        chunk_size,
        levels: target_levels.clone(),
        compression: target_compression,
    };

    let mut target_backend = B::create(output_path, target_metadata)?;

    for &(source_level, target_level) in &level_mapping {
        println!("\nConverting Level {source_level} -> {target_level}...");

        let (source_chunk_level_u32, source_chunk_ids) = source_backend.chunk_ids_for_level(source_level)?;
        if source_chunk_ids.is_empty() {
            println!("  Warning: source level {source_level} has no chunk IDs, skipping.");
            continue;
        }

        let mut min_lon = f64::INFINITY;
        let mut max_lon = f64::NEG_INFINITY;
        let mut min_lat = f64::INFINITY;
        let mut max_lat = f64::NEG_INFINITY;
        let mut has_any = false;

        for chunk_id_str in &source_chunk_ids {
            let chunk_zone_id = ZoneId::from_str(chunk_id_str)?;
            let zones = source_grid.zone_from_id(chunk_zone_id, Some(DggrsApiConfig {
                region: true,
                center: false,
                vertex_count: false,
                children: false,
                neighbors: false,
                area_sqm: false,
                densify: false,
            }))?;
            if let Some(zone) = zones.zones.first() {
                if let Some(region) = &zone.region {
                    for pt in &region.exterior {
                        min_lon = min_lon.min(pt.lon);
                        max_lon = max_lon.max(pt.lon);
                        min_lat = min_lat.min(pt.lat);
                        max_lat = max_lat.max(pt.lat);
                        has_any = true;
                    }
                }
            }
        }

        let bbox = if has_any {
            let lon_padding = (max_lon - min_lon) * 0.01;
            let lat_padding = (max_lat - min_lat) * 0.01;
            let min_lon = (min_lon - lon_padding).max(-180.0);
            let max_lon = (max_lon + lon_padding).min(180.0);
            let min_lat = (min_lat - lat_padding).max(-90.0);
            let max_lat = (max_lat + lat_padding).min(90.0);

            let tolerance = 1e-4;
            let is_global = (min_lon <= -180.0 + tolerance)
                && (max_lon >= 180.0 - tolerance)
                && (min_lat <= -90.0 + tolerance)
                && (max_lat >= 90.0 - tolerance);
            if is_global {
                None
            } else {
                Some(BoundingBox::new(min_lon, min_lat, max_lon, max_lat))
            }
        } else {
            None
        };

        let (target_chunk_level, target_level_chunk_size) = choose_best_chunk_level_and_size(
            RefinementLevel::new(target_level as i32)?,
            target_min_chunk_level,
            target_max_relative_depth_allowed,
            target_dggrs.spec().aperture as u64,
            data_type_size_bytes,
        )?;

        let target_chunk_zones = target_grid.zones_from_bbox(target_chunk_level, bbox, Some(CONFIG))?;
        if target_chunk_zones.zones.is_empty() {
            return Err(EncodingError::Grid(
                "no target zones found intersecting bounding box".into(),
            ));
        }
        let target_chunk_ids: Vec<String> = target_chunk_zones
            .zones
            .iter()
            .map(|z| z.id.to_string())
            .collect();

        target_backend.set_level_chunk_ids(
            target_level,
            target_chunk_level.get() as u32,
            target_chunk_ids.clone(),
        )?;

        let encoded_num_cells = (target_chunk_ids.len() as u64)
            .checked_mul(target_level_chunk_size)
            .ok_or_else(|| EncodingError::Storage("encoded cell count overflow".into()))?;

        let band_count = source_backend.band_count();
        for band_idx in 0..band_count {
            target_backend.create_level(
                target_level,
                band_idx,
                encoded_num_cells,
                target_level_chunk_size,
            )?;
        }

        let source_chunk_level = RefinementLevel::new(source_chunk_level_u32 as i32)?;
        let source_relative_depth = RelativeDepth::new(source_level as i32 - source_chunk_level.get())?;

        println!("  Building source cell lookup index...");
        let mut source_cell_to_index = HashMap::new();
        for (chunk_idx, chunk_id_str) in source_chunk_ids.iter().enumerate() {
            let chunk_zone_id = ZoneId::from_str(chunk_id_str)?;
            let children = source_grid.zones_from_parent(
                source_relative_depth,
                chunk_zone_id,
                Some(ID_ONLY_CONFIG),
            )?;
            for (in_chunk_idx, child) in children.zones.iter().enumerate() {
                source_cell_to_index.insert(child.id.clone(), (chunk_idx, in_chunk_idx));
            }
        }
        let fill_value_bytes: Vec<Vec<u8>> = target_backend
            .metadata()
            .attributes
            .iter()
            .map(|attr| {
                let fill_val = parse_fill_value_to_f64(&attr.dtype, &attr.fill_value)?;
                encode_value_from_f64(&attr.dtype, fill_val)
            })
            .collect::<Result<_, EncodingError>>()?;

        let total_target_chunks = target_chunk_ids.len();
        let chunk_progress = ProgressBar::new(total_target_chunks as u64);
        let style = ProgressStyle::with_template(
            "  Resampling chunks [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)",
        )
        .map_err(|e| EncodingError::Storage(format!("invalid progress bar template: {e}")))?
        .progress_chars("=> ");
        chunk_progress.set_style(style);

        let progress_counter = std::sync::atomic::AtomicU64::new(0);
        let target_relative_depth = RelativeDepth::new(target_level as i32 - target_chunk_level.get())?;
        let center_config = DggrsApiConfig {
            center: true,
            ..CONFIG
        };

        target_chunk_ids
            .par_iter()
            .enumerate()
            .try_for_each(|(target_chunk_idx, target_chunk_id_str)| -> Result<(), EncodingError> {
                let target_chunk_zone_id = ZoneId::from_str(target_chunk_id_str)?;
                let children = target_grid.zones_from_parent(
                    target_relative_depth,
                    target_chunk_zone_id,
                    Some(center_config),
                )?;

                if children.zones.len() > target_level_chunk_size as usize {
                    return Err(EncodingError::Grid(format!(
                        "target chunk {target_chunk_id_str} has {} children but chunk_size is {target_level_chunk_size}",
                        children.zones.len()
                    )));
                }

                let mut band_chunk_bytes: Vec<Vec<u8>> = (0..band_count)
                    .map(|band_idx| {
                        let dtype = &target_backend.metadata().attributes[band_idx as usize].dtype;
                        let val_size = dtype.byte_size();
                        let fill_bytes = &fill_value_bytes[band_idx as usize];
                        let mut chunk_bytes = vec![0_u8; target_level_chunk_size as usize * val_size];
                        for cell_idx in 0..target_level_chunk_size as usize {
                            let start = cell_idx * val_size;
                            let end = start + val_size;
                            chunk_bytes[start..end].copy_from_slice(fill_bytes);
                        }
                        chunk_bytes
                    })
                    .collect();

                let source_refinement_level = RefinementLevel::new(source_level as i32)?;

                let mut required_src_chunks = HashSet::new();
                for target_cell in &children.zones {
                    let center = target_cell.center.ok_or_else(|| {
                        EncodingError::Grid(format!(
                            "target zone {} has no center coordinates",
                            target_cell.id
                        ))
                    })?;

                    let source_zones = source_grid.zone_from_point(
                        source_refinement_level,
                        center,
                        Some(ID_ONLY_CONFIG),
                    )?;

                    if let Some(source_zone) = source_zones.zones.first() {
                        if let Some(&(src_chunk_idx, _)) = source_cell_to_index.get(&source_zone.id) {
                            required_src_chunks.insert(src_chunk_idx);
                        }
                    }
                }

                let mut local_src_chunks = HashMap::with_capacity(required_src_chunks.len());
                for &src_chunk_idx in &required_src_chunks {
                    let mut bands_data = Vec::with_capacity(band_count as usize);
                    for band_idx in 0..band_count {
                        let chunk = source_backend.read_chunk(source_level, band_idx, src_chunk_idx as u64)?;
                        bands_data.push(chunk);
                    }
                    local_src_chunks.insert(src_chunk_idx, bands_data);
                }

                for (in_chunk_idx, target_cell) in children.zones.iter().enumerate() {
                    let center = target_cell.center.ok_or_else(|| {
                        EncodingError::Grid(format!(
                            "target zone {} has no center coordinates",
                            target_cell.id
                        ))
                    })?;

                    let source_zones = source_grid.zone_from_point(
                        source_refinement_level,
                        center,
                        Some(ID_ONLY_CONFIG),
                    )?;

                    if let Some(source_zone) = source_zones.zones.first() {
                        if let Some(&(src_chunk_idx, src_in_chunk_idx)) = source_cell_to_index.get(&source_zone.id) {
                            if let Some(bands_data) = local_src_chunks.get(&src_chunk_idx) {
                                for band_idx in 0..band_count {
                                    let dtype = &target_backend.metadata().attributes[band_idx as usize].dtype;
                                    let val_size = dtype.byte_size();
                                    let src_chunk = &bands_data[band_idx as usize];

                                    let src_start = src_in_chunk_idx * val_size;
                                    let src_end = src_start + val_size;
                                    if src_chunk.len() < src_end {
                                        return Err(EncodingError::Storage(format!(
                                            "source chunk {src_chunk_idx} is too small"
                                        )));
                                    }

                                    let target_start = in_chunk_idx * val_size;
                                    let target_end = target_start + val_size;
                                    band_chunk_bytes[band_idx as usize][target_start..target_end]
                                        .copy_from_slice(&src_chunk[src_start..src_end]);
                                }
                            }
                        }
                    }
                }

                for band_idx in 0..band_count {
                    target_backend.write_chunk(
                        target_level,
                        band_idx,
                        target_chunk_idx as u64,
                        &band_chunk_bytes[band_idx as usize],
                    )?;
                }

                let done = progress_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                chunk_progress.set_position(done);

                Ok(())
            })?;

        chunk_progress.finish_with_message("  Resampling complete");
    }

    Ok(target_backend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdal::DriverManager;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::path::PathBuf;
    use crate::zarr::ZarrBackend;
    use crate::query::query_value_for_point;

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("gp_encoding_{name}_{nanos}"))
    }

    #[test]
    fn test_attribute_schema_uses_declared_no_data() {
        let driver = DriverManager::get_driver_by_name("MEM").expect("MEM driver");
        let dataset = driver
            .create_with_band_type::<f32, _>("", 1, 1, 1)
            .expect("create in-memory raster");
        let mut band = dataset.rasterband(1).expect("raster band");
        band.set_no_data_value(Some(-9999.0)).expect("set no-data");

        let schema = attribute_schema_from_band(&band).expect("attribute schema");

        assert_eq!(schema.fill_value, "-9999");
    }

    #[test]
    fn test_attribute_schema_uses_default_when_no_data_is_missing() {
        let driver = DriverManager::get_driver_by_name("MEM").expect("MEM driver");
        let dataset = driver
            .create_with_band_type::<f32, _>("", 1, 1, 1)
            .expect("create in-memory raster");
        let band = dataset.rasterband(1).expect("raster band");

        let schema = attribute_schema_from_band(&band).expect("attribute schema");

        assert_eq!(schema.fill_value, "NaN");
    }

    #[test]
    fn test_convert_dggrs_store_to_backend_resamples_correctly() {
        let src_store_path = unique_temp_dir("dggrs_convert_src");
        let tgt_store_path = unique_temp_dir("dggrs_convert_tgt");

        // 1. Create a source Zarr store (H3)
        let dggrs_src = DggrsUid::H3;
        let grid_src = get(dggrs_src).expect("resolve src dggrs");
        let src_refinement_level = RefinementLevel::new(1).expect("src refinement level");
        let src_chunk_level = RefinementLevel::new(0).expect("src chunk level");
        let src_chunk_size = u64::from(dggrs_src.spec().aperture);

        let src_chunk_zones = grid_src
            .zones_from_bbox(src_chunk_level, None, Some(ID_ONLY_CONFIG))
            .expect("src chunk zones");
        assert!(!src_chunk_zones.zones.is_empty(), "expected source chunk zones");

        let src_chunk0_id = src_chunk_zones.zones[0].id.clone();
        
        let mut child_config = ID_ONLY_CONFIG;
        child_config.center = true;
        
        let src_children = grid_src
            .zones_from_parent(RelativeDepth::new_const(1), src_chunk0_id.clone(), Some(child_config))
            .expect("src children");
        assert!(!src_children.zones.is_empty(), "expected source children");

        let src_metadata = DatasetMetadata {
            dggrs: dggrs_src,
            attributes: vec![AttributeSchema {
                dtype: DataType::Float32,
                fill_value: "0.0".to_string(),
            }],
            chunk_size: src_chunk_size,
            levels: vec![src_refinement_level.get() as u32],
            compression: None,
        };

        let mut src_backend = ZarrBackend::create(&src_store_path, src_metadata).expect("create src zarr");
        src_backend
            .set_level_chunk_ids(
                src_refinement_level.get() as u32,
                src_chunk_level.get() as u32,
                vec![src_chunk0_id.to_string()],
            )
            .expect("set src chunk ids");
        src_backend
            .create_level(src_refinement_level.get() as u32, 0, src_chunk_size, src_chunk_size)
            .expect("create src level");

        let chunk_values = vec![42.0_f32; src_chunk_size as usize];
        let chunk_bytes: Vec<u8> = chunk_values
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect();
        src_backend
            .write_chunk(src_refinement_level.get() as u32, 0, 0, &chunk_bytes)
            .expect("write src chunk");

        // Use the center point of one child cell
        let target_center_pt = src_children.zones[0].center.expect("expected center point");

        // 2. Convert from H3 to IVEA3H
        let dggrs_tgt = DggrsUid::IVEA3H;
        let tgt_backend: ZarrBackend = convert_dggrs_store_to_backend(
            &src_store_path,
            &tgt_store_path,
            dggrs_tgt,
            None,
        )
        .expect("convert dggrs store");

        assert_eq!(tgt_backend.metadata().dggrs, dggrs_tgt);
        assert_eq!(tgt_backend.metadata().attributes.len(), 1);
        assert_eq!(tgt_backend.metadata().attributes[0].dtype, DataType::Float32);

        // 3. Query a point in the target backend to see if it resampled correctly
        let tgt_levels = tgt_backend.levels();
        assert!(!tgt_levels.is_empty(), "target levels should not be empty");
        let tgt_level = RefinementLevel::new(tgt_levels[0] as i32).expect("tgt level");

        let res_bytes = query_value_for_point(&tgt_backend, tgt_level, 0, target_center_pt)
            .expect("query value on target store");
        let value = f32::from_ne_bytes([res_bytes[0], res_bytes[1], res_bytes[2], res_bytes[3]]);
        
        assert_eq!(value, 42.0);

        let _ = std::fs::remove_dir_all(&src_store_path);
        let _ = std::fs::remove_dir_all(&tgt_store_path);
    }

    #[test]
    fn test_get_subdataset_short_name() {
        assert_eq!(get_subdataset_short_name("NETCDF:\"file.nc\":elevation"), "elevation");
        assert_eq!(get_subdataset_short_name("HDF5:\"file.h5\"://elevation"), "elevation");
        assert_eq!(get_subdataset_short_name("NETCDF:\"/path/to/file.nc\":temp"), "temp");
        assert_eq!(get_subdataset_short_name("simple_name"), "simple_name");
        assert_eq!(get_subdataset_short_name("HDF5:file.h5:variable_with_spaces "), "variable_with_spaces");
    }
}
