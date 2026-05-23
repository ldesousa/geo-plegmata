use std::path::Path;

use gdal::raster::GdalDataType;
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::{Dataset, GeoTransformEx};
use geoplegma::api::DggrsApiConfig;
use geoplegma::get;
use geoplegma::types::{BoundingBox, DggrsUid, Point, RefinementLevel, RelativeDepth};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::AttributeSchema;
use crate::common::CONFIG;
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

fn get_corners_and_pixel_size(
    dataset: &Dataset,
) -> Result<(Option<BoundingBox>, f64, f64), EncodingError> {
    let (width_px, height_px) = dataset.raster_size();
    let w = width_px as f64;
    let h = height_px as f64;

    let gt = dataset.geo_transform()?;
    let src_srs = dataset.spatial_ref()?;

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
        return Err(EncodingError::GeoTiff(
            "raster has zero width or height".into(),
        ));
    }

    let mut xs = vec![center.lon];
    let mut ys = vec![center.lat];
    let mut zs = vec![];
    wgs84_to_src.transform_coords(&mut xs, &mut ys, &mut zs)?;

    let det = gt[1] * gt[5] - gt[2] * gt[4];
    if det.abs() < f64::EPSILON {
        return Err(EncodingError::GeoTiff(
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
        return Err(EncodingError::GeoTiff(
            "geotransform has zero pixel size".into(),
        ));
    }

    let world_pixel_count =
        ((40_075_016.685 / pixel_width) * (40_075_016.685 / pixel_height)) as u64; // TODO: support different projections?

    println!("world pixel count: {}", world_pixel_count);
    let mut best_level: Option<RefinementLevel> = None;
    let mut best_diff = u64::MAX;

    let min_level = grid.min_refinement_level()?;
    let max_level = grid.max_refinement_level()?;

    for raw_level in min_level.get()..=max_level.get() {
        let level = RefinementLevel::new_const(raw_level);

        let zone_count = grid.zone_count(level)?;

        let diff = world_pixel_count.abs_diff(zone_count);

        if diff < best_diff {
            best_diff = diff;
            best_level = Some(level);
        }
    }
    let diff_percentage = (best_diff as f64 / world_pixel_count as f64) * 100.0;
    println!(
        "best level: {} with diff {}",
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

pub fn convert_geotiff_file_to_backend<B>(
    geotiff_path: &Path,
    output_path: &Path,
    dggrs: DggrsUid,
    compression: Option<Compression>,
) -> Result<(B, SourceRasterReport, ConversionReport), EncodingError>
where
    B: StorageBackend,
{
    if !geotiff_path.exists() {
        return Err(EncodingError::GeoTiff(format!(
            "input GeoTIFF does not exist: {}",
            geotiff_path.display()
        )));
    }
    if !geotiff_path.is_file() {
        return Err(EncodingError::GeoTiff(format!(
            "input GeoTIFF is not a file: {}",
            geotiff_path.display()
        )));
    }
    let grid = get(dggrs)?;

    let dataset = Dataset::open(geotiff_path)?;
    let source_report = compute_source_report(&dataset)?;

    let bands = dataset
        .rasterbands()
        .map(|b| b.map(|band| band.band_type()))
        .collect::<Result<Vec<_>, _>>()?;
    let metadata_bands = bands
        .iter()
        .map(|band_type| {
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
                    return Err(EncodingError::GeoTiff(format!(
                        "unsupported GDAL data type: {band_type:?}"
                    )));
                }
            };

            Ok(AttributeSchema {
                dtype,
                fill_value: Some("0.0".to_string()),
            })
        })
        .collect::<Result<Vec<_>, EncodingError>>()?;

    if metadata_bands.is_empty() {
        return Err(EncodingError::Storage(
            "dataset metadata must define at least one attribute".into(),
        ));
    }

    let (width, height) = dataset.raster_size();

    if width == 0 || height == 0 {
        return Err(EncodingError::GeoTiff(
            "raster has zero width or height".into(),
        ));
    }

    let gt = dataset.geo_transform()?;
    let src_srs = dataset.spatial_ref()?;
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
            let fill_val = match &attr.fill_value {
                Some(value) => parse_fill_value_to_f64(&attr.dtype, value)?,
                None => 0.0,
            };
            encode_value_from_f64(&attr.dtype, fill_val)
        })
        .collect::<Result<_, EncodingError>>()?;

    let progress_counter = std::sync::atomic::AtomicU64::new(0);

    let geotiff_path_buf = geotiff_path.to_path_buf();

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
                let thread_dataset = Dataset::open(&geotiff_path_buf)?;
                Ok((transform, thread_dataset))
            },
            |state, (chunk_index, chunk_zone)| {
                let (wgs84_to_src, thread_dataset) = match state {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(EncodingError::GeoTiff(format!(
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
                            _ => return Err(EncodingError::GeoTiff(format!(
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
