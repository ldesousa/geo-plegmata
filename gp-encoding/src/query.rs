// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::BTreeMap;
use std::str::FromStr;

use geoplegma::get;
use geoplegma::types::{DggrsUid, Point, RefinementLevel, RelativeDepth, ZoneId};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use serde::ser::{SerializeSeq, Serializer};
use serde_json::Value;

use crate::common::{CENTER_CONFIG, ID_ONLY_CONFIG};
use crate::error::EncodingError;
use crate::storage::StorageBackend;
use crate::value::{
    decode_value_to_f64, decode_value_to_json, parse_fill_value_to_f64, parse_fill_value_to_json,
};

#[derive(Debug, Clone, Serialize)]
pub struct VisualizationCell {
    pub polygon: Vec<[f64; 2]>,
    #[serde(flatten)]
    pub bands: BTreeMap<String, Value>,
}

/// Query a cell value by geographic point.
///
/// The point is resolved to a DGGS cell at `refinement_level`, then the
/// corresponding value bytes are retrieved from the storage backend.
pub fn query_value_for_point<B: StorageBackend>(
    backend: &B,
    refinement_level: RefinementLevel,
    band: u32,
    point: Point,
) -> Result<Vec<u8>, EncodingError> {
    let grid = get(backend.metadata().dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve DGGS: {e}")))?;

    let zones = grid
        .zone_from_point(refinement_level, point, Some(ID_ONLY_CONFIG))
        .map_err(|e| EncodingError::Grid(e.to_string()))?;

    let zone = zones
        .zones
        .first()
        .ok_or_else(|| EncodingError::Grid("zone_from_point returned no zones".into()))?;

    let level = u32::try_from(refinement_level.get()).map_err(|_| {
        EncodingError::Grid(format!(
            "refinement level {} cannot be represented as u32",
            refinement_level.get()
        ))
    })?;

    println!(
        "Resolved point ({}, {}) to zone ID {:?} at level {} and band {}",
        point.lon, point.lat, zone.id, level, band
    );

    query_value_by_cell_index(backend, level, band, &zone.id)
}

/// Query a single cell value by linearized cell index.
///
/// Returns the raw bytes for one attribute value according to the dataset
/// attribute schema.
pub fn query_value_by_cell_index<B: StorageBackend>(
    backend: &B,
    level: u32,
    band: u32,
    zone_id: &ZoneId,
) -> Result<Vec<u8>, EncodingError> {
    let (chunk_level_u32, chunk_ids) = backend.chunk_ids_for_level(level)?;
    let cell_index = zone_id.to_string();
    let value_size = backend
        .metadata()
        .attributes
        .first()
        .map(|attr| attr.dtype.byte_size())
        .ok_or_else(|| {
            EncodingError::Storage("dataset metadata must define at least one attribute".into())
        })?;

    let grid = get(backend.metadata().dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve DGGS: {e}")))?;

    let refinement_level = RefinementLevel::new(i32::try_from(level).map_err(|_| {
        EncodingError::Grid(format!("level {level} cannot be represented as i32"))
    })?)?;

    let chunk_level = RefinementLevel::new(chunk_level_u32 as i32)?;
    let depth_i32 = refinement_level.get() - chunk_level.get();

    if depth_i32 < 0 {
        return Err(EncodingError::Storage(format!(
            "level {} is below stored chunk level {}",
            refinement_level.get(),
            chunk_level.get()
        )));
    }

    let mut chunk_zone_id = zone_id.clone();
    for _ in 0..depth_i32 {
        let parent = grid
            .primary_parent_from_zone(chunk_zone_id.clone(), Some(ID_ONLY_CONFIG))
            .map_err(|e| EncodingError::Grid(e.to_string()))?;
        let parent_zone = parent.zones.first().ok_or_else(|| {
            EncodingError::Grid("primary_parent_from_zone returned no zones".into())
        })?;
        chunk_zone_id = parent_zone.id.clone();
    }

    let chunk_id = chunk_zone_id.to_string();

    let chunk_index = chunk_ids
        .iter()
        .position(|id| id == &chunk_id)
        .ok_or_else(|| {
            EncodingError::Storage(format!(
                "zone {cell_index} belongs to chunk id {chunk_id}, which is not present in dataset metadata"
            ))
        })? as u64;

    let relative_depth = RelativeDepth::new(refinement_level.get() - chunk_level.get())?;
    let children = grid
        .zones_from_parent(relative_depth, chunk_zone_id, Some(ID_ONLY_CONFIG))
        .map_err(|e| EncodingError::Grid(e.to_string()))?;

    let in_chunk_index = children
        .zones
        .iter()
        .position(|child| child.id == *zone_id)
        .ok_or_else(|| {
            EncodingError::Storage(format!(
                "zone {cell_index} is not present in computed child list for chunk {chunk_id}"
            ))
        })?;

    let chunk = backend.read_chunk(level, band, chunk_index)?;

    let start = in_chunk_index * value_size;
    let end = start + value_size;
    if chunk.len() < end {
        return Err(EncodingError::Storage(format!(
            "chunk {chunk_index} at level {level} is too small for cell index {cell_index}"
        )));
    }

    Ok(chunk[start..end].to_vec())
}

pub fn export_level_as_visualization_json<B: StorageBackend>(
    backend: &B,
    level: u32,
) -> Result<Vec<VisualizationCell>, EncodingError> {
    let mut rows = Vec::new();
    visit_level_as_visualization_cells(backend, level, |cell| {
        rows.push(cell);
        Ok(())
    })?;

    Ok(rows)
}

pub fn export_level_as_visualization_binary<B: StorageBackend>(
    backend: &B,
    level: u32,
    bbox: Option<geoplegma::types::BoundingBox>,
) -> Result<Vec<u8>, EncodingError> {
    let band_count = backend.band_count();
    let mut output = Vec::new();
    output.extend_from_slice(&(band_count as u32).to_le_bytes());
    output.extend_from_slice(&0_u32.to_le_bytes());

    let mut cell_count = 0_u32;
    visit_level_as_visualization_cells_f64(backend, level, bbox, |region, values| {
        let coords_count = u16::try_from(region.exterior.len()).map_err(|_| {
            EncodingError::Storage(format!(
                "too many vertices in region: {}",
                region.exterior.len()
            ))
        })?;
        output.extend_from_slice(&coords_count.to_le_bytes());
        for pt in region.exterior {
            output.extend_from_slice(&pt.lon.to_le_bytes());
            output.extend_from_slice(&pt.lat.to_le_bytes());
        }
        for value in values {
            output.extend_from_slice(&value.to_le_bytes());
        }
        cell_count = cell_count.saturating_add(1);
        Ok(())
    })?;

    output[4..8].copy_from_slice(&cell_count.to_le_bytes());
    Ok(output)
}

pub fn write_level_as_visualization_json<B: StorageBackend, W: std::io::Write>(
    backend: &B,
    level: u32,
    writer: W,
) -> Result<usize, EncodingError> {
    let mut serializer = serde_json::Serializer::new(writer);
    let mut sequence = serializer
        .serialize_seq(None)
        .map_err(|e: serde_json::Error| EncodingError::Storage(e.to_string()))?;

    let cell_count = visit_level_as_visualization_cells(backend, level, |cell| {
        sequence
            .serialize_element(&cell)
            .map_err(|e: serde_json::Error| EncodingError::Storage(e.to_string()))
    })?;

    sequence
        .end()
        .map_err(|e: serde_json::Error| EncodingError::Storage(e.to_string()))?;

    Ok(cell_count)
}

fn visit_level_as_visualization_cells<B: StorageBackend, F>(
    backend: &B,
    level: u32,
    mut visit_cell: F,
) -> Result<usize, EncodingError>
where
    F: FnMut(VisualizationCell) -> Result<(), EncodingError>,
{

    let band_count = backend.band_count();
    if band_count == 0 {
        return Err(EncodingError::Storage(
            "dataset metadata must define at least one band".into(),
        ));
    }

    let (chunk_level_u32, chunk_ids) = backend.chunk_ids_for_level(level)?;
    let target_level = RefinementLevel::new(
        i32::try_from(level)
            .map_err(|_| EncodingError::Storage(format!("level {level} cannot fit i32")))?,
    )?;

    let chunk_level = RefinementLevel::new(chunk_level_u32 as i32)?;
    if target_level.get() < chunk_level.get() {
        return Err(EncodingError::Storage(format!(
            "level {} is below stored chunk level {}",
            target_level.get(),
            chunk_level.get()
        )));
    }

    let relative_depth = RelativeDepth::new(target_level.get() - chunk_level.get())?;
    let grid = get(backend.metadata().dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve DGGS: {e}")))?;

    let fill_values: Vec<Option<Value>> = backend
        .metadata()
        .attributes
        .iter()
        .map(|attr| {
            if let Some(fill_value) = &attr.fill_value {
                Ok(Some(parse_fill_value_to_json(&attr.dtype, fill_value)?))
            } else {
                Ok(None)
            }
        })
        .collect::<Result<_, EncodingError>>()?;

    let mut row_count = 0_usize;
    let chunk_progress = ProgressBar::new(chunk_ids.len() as u64);
    let style = ProgressStyle::with_template(
        "processing chunks [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)",
    )
    .map_err(|e| EncodingError::Storage(format!("invalid progress bar template: {e}")))?
    .progress_chars("=> ");
    chunk_progress.set_style(style);

    for (chunk_index, chunk_id) in chunk_ids.iter().enumerate() {
        chunk_progress.set_message(format!("chunk {chunk_index} {chunk_id}"));

        let chunk_zone_id = ZoneId::from_str(chunk_id)
            .map_err(|e| EncodingError::Storage(format!("invalid chunk id '{chunk_id}': {e}")))?;

        let mut config = ID_ONLY_CONFIG;
        config.region = true;

        let children = grid
            .zones_from_parent(relative_depth, chunk_zone_id, Some(config))
            .map_err(|e| EncodingError::Grid(e.to_string()))?;

        let chunks_for_bands: Vec<Vec<u8>> = (0..band_count)
            .map(|band| backend.read_chunk(level, band, chunk_index as u64))
            .collect::<Result<Vec<_>, _>>()?;

        for (in_chunk_index, zone) in children.zones.iter().enumerate() {
            let mut bands = BTreeMap::new();
            let mut has_non_fill = false;

            for band in 0..band_count {
                let dtype = &backend.metadata().attributes[band as usize].dtype;
                let value_size = dtype.byte_size();
                let start = in_chunk_index * value_size;
                let end = start + value_size;
                let chunk = &chunks_for_bands[band as usize];

                if chunk.len() < end {
                    return Err(EncodingError::Storage(format!(
                        "chunk {chunk_index} at level {level} is too small for child index {in_chunk_index} and band {band}"
                    )));
                }

                let value = decode_value_to_json(dtype, &chunk[start..end])?;
                if fill_values[band as usize]
                    .as_ref()
                    .is_some_and(|fill_value| *fill_value == value)
                {
                    continue;
                }
                bands.insert(format!("band_{band}"), value);
                has_non_fill = true;
            }

            if !has_non_fill {
                continue;
            }

            let region = zone.region.as_ref().ok_or_else(|| EncodingError::Storage("zone missing region".into()))?;
            visit_cell(VisualizationCell {
                polygon: region.exterior.iter().map(|p| [p.lon, p.lat]).collect(),
                bands,
            })?;
            row_count += 1;
        }

        chunk_progress.inc(1);
    }

    chunk_progress.finish_with_message("processing chunks [done]");

    Ok(row_count)
}

fn visit_level_as_visualization_cells_f64<B: StorageBackend, F>(
    backend: &B,
    level: u32,
    bbox: Option<geoplegma::types::BoundingBox>,
    mut visit_cell: F,
) -> Result<usize, EncodingError>
where
    F: FnMut(geoplegma::types::Region, Vec<f64>) -> Result<(), EncodingError>,
{

    let band_count = backend.band_count();
    if band_count == 0 {
        return Err(EncodingError::Storage(
            "dataset metadata must define at least one band".into(),
        ));
    }

    let (chunk_level_u32, chunk_ids) = backend.chunk_ids_for_level(level)?;
    let target_level = RefinementLevel::new(
        i32::try_from(level)
            .map_err(|_| EncodingError::Storage(format!("level {level} cannot fit i32")))?,
    )?;

    let chunk_level = RefinementLevel::new(chunk_level_u32 as i32)?;
    if target_level.get() < chunk_level.get() {
        return Err(EncodingError::Storage(format!(
            "level {} is below stored chunk level {}",
            target_level.get(),
            chunk_level.get()
        )));
    }

    let relative_depth = RelativeDepth::new(target_level.get() - chunk_level.get())?;
    let grid = get(backend.metadata().dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve DGGS: {e}")))?;

    let fill_values: Vec<Option<f64>> = backend
        .metadata()
        .attributes
        .iter()
        .map(|attr| {
            if let Some(fill_value) = &attr.fill_value {
                Ok(Some(parse_fill_value_to_f64(&attr.dtype, fill_value)?))
            } else {
                Ok(None)
            }
        })
        .collect::<Result<_, EncodingError>>()?;

    let mut row_count = 0_usize;

    let mut chunk_ids_to_process: Vec<(usize, String)> = chunk_ids
        .into_iter()
        .enumerate()
        .collect();
    
    let effective_bbox = if level <= 2 { None } else { bbox };

    if let Some(bounds) = effective_bbox {
        let chunk_zones = grid
            .zones_from_bbox(chunk_level, Some(bounds), Some(ID_ONLY_CONFIG))
            .map_err(|e| EncodingError::Grid(e.to_string()))?;
        let valid_chunk_ids: std::collections::HashSet<String> = chunk_zones
            .zones
            .into_iter()
            .map(|z| z.id.to_string())
            .collect();
        chunk_ids_to_process.retain(|(_, id)| valid_chunk_ids.contains(id));
    }

    let chunk_progress = ProgressBar::new(chunk_ids_to_process.len() as u64);
    let style = ProgressStyle::with_template(
        "processing chunks [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)",
    )
    .map_err(|e| EncodingError::Storage(format!("invalid progress bar template: {e}")))?
    .progress_chars("=> ");
    chunk_progress.set_style(style);

    let mut config = if effective_bbox.is_some() { CENTER_CONFIG } else { ID_ONLY_CONFIG };
    config.region = true;

    for (progress_index, (chunk_index, chunk_id)) in chunk_ids_to_process.into_iter().enumerate() {
        chunk_progress.set_message(format!("chunk {progress_index} {chunk_id}"));

        let chunk_zone_id = ZoneId::from_str(&chunk_id)
            .map_err(|e| EncodingError::Storage(format!("invalid chunk id '{chunk_id}': {e}")))?;

        let children = grid
            .zones_from_parent(relative_depth, chunk_zone_id, Some(config))
            .map_err(|e| EncodingError::Grid(e.to_string()))?;

        let chunks_for_bands: Vec<Vec<u8>> = (0..band_count)
            .map(|band| backend.read_chunk(level, band, chunk_index as u64))
            .collect::<Result<Vec<_>, _>>()?;

        for (in_chunk_index, zone) in children.zones.iter().enumerate() {
            if let (Some(bounds), Some(center)) = (effective_bbox, zone.center.as_ref()) {
                if center.lon < bounds.min_lon
                    || center.lon > bounds.max_lon
                    || center.lat < bounds.min_lat
                    || center.lat > bounds.max_lat
                {
                    continue;
                }
            }

            let mut values = vec![f64::NAN; band_count as usize];
            let mut has_non_fill = false;

            for band in 0..band_count {
                let dtype = &backend.metadata().attributes[band as usize].dtype;
                let value_size = dtype.byte_size();
                let start = in_chunk_index * value_size;
                let end = start + value_size;
                let chunk = &chunks_for_bands[band as usize];

                if chunk.len() < end {
                    return Err(EncodingError::Storage(format!(
                        "chunk {chunk_index} at level {level} is too small for child index {in_chunk_index} and band {band}"
                    )));
                }

                let value = decode_value_to_f64(dtype, &chunk[start..end])?;
                if fill_values[band as usize]
                    .as_ref()
                    .is_some_and(|fill_value| *fill_value == value)
                {
                    continue;
                }

                values[band as usize] = value;
                has_non_fill = true;
            }

            if !has_non_fill {
                continue;
            }

            let region = zone.region.clone().ok_or_else(|| EncodingError::Storage("zone missing region".into()))?;
            visit_cell(region, values)?;
            row_count += 1;
        }

        chunk_progress.inc(1);
    }

    chunk_progress.finish_with_message("processing chunks [done]");

    Ok(row_count)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use geoplegma::get;
    use geoplegma::types::{DggrsUid, RefinementLevel, RelativeDepth};

    use crate::common::ID_ONLY_CONFIG;
    use crate::models::{AttributeSchema, DataType, DatasetMetadata};
    use crate::query::query_value_by_cell_index;
    use crate::storage::StorageBackend;
    use crate::zarr::ZarrBackend;

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("gp_encoding_{name}_{nanos}"))
    }

    #[test]
    fn query_value_bytes_by_cell_index_reads_expected_cell() {
        let store_path = unique_temp_dir("query_test");

        let dggrs = DggrsUid::H3;
        let grid = get(dggrs).expect("resolve dggrs");

        let min_level = grid.min_refinement_level().expect("min level").get();
        let refinement_level =
            RefinementLevel::new((min_level + 1).max(1)).expect("refinement level");
        let chunk_level = RefinementLevel::new(refinement_level.get() - 1).expect("chunk level");
        let chunk_size = u64::from(dggrs.spec().aperture);

        let chunk_zones = grid
            .zones_from_bbox(chunk_level, None, Some(ID_ONLY_CONFIG))
            .expect("chunk zones");
        assert!(
            chunk_zones.zones.len() >= 2,
            "expected at least 2 chunk zones"
        );

        let chunk0 = chunk_zones.zones[0].id.clone();
        let chunk1 = chunk_zones.zones[1].id.clone();

        let children0 = grid
            .zones_from_parent(RelativeDepth::new_const(1), chunk0.clone(), Some(ID_ONLY_CONFIG))
            .expect("children for chunk0");
        let children1 = grid
            .zones_from_parent(RelativeDepth::new_const(1), chunk1.clone(), Some(ID_ONLY_CONFIG))
            .expect("children for chunk1");

        assert!(
            children0.zones.len() >= 2,
            "expected at least 2 children in chunk0"
        );
        assert!(!children1.zones.is_empty(), "expected children in chunk1");

        let cell0 = children0.zones[0].id.clone();
        let cell1 = children0.zones[1].id.clone();
        let cell2 = children1.zones[0].id.clone();

        let metadata = DatasetMetadata {
            dggrs,
            attributes: vec![AttributeSchema {
                dtype: DataType::UInt16,
                fill_value: None,
            }],
            chunk_size,
            levels: vec![refinement_level.get() as u32],
            compression: None,
        };

        let mut backend = ZarrBackend::create(&store_path, metadata).expect("create zarr");
        backend
            .set_level_chunk_ids(
                refinement_level.get() as u32,
                chunk_level.get() as u32,
                vec![chunk0.to_string(), chunk1.to_string()],
            )
            .expect("set chunk ids");
        backend
            .create_level(refinement_level.get() as u32, 0, 2 * chunk_size, chunk_size)
            .expect("create level");

        let mut chunk0_values = vec![0_u16; chunk_size as usize];
        chunk0_values[0] = 10;
        chunk0_values[1] = 20;

        let mut chunk1_values = vec![0_u16; chunk_size as usize];
        chunk1_values[0] = 60;

        let chunk0_bytes: Vec<u8> = chunk0_values
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect();
        let chunk1_bytes: Vec<u8> = chunk1_values
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect();

        backend
            .write_chunk(refinement_level.get() as u32, 0, 0, &chunk0_bytes)
            .expect("write chunk0");
        backend
            .write_chunk(refinement_level.get() as u32, 0, 1, &chunk1_bytes)
            .expect("write chunk1");

        let bytes = query_value_by_cell_index(&backend, refinement_level.get() as u32, 0, &cell0)
            .expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 10);

        let bytes = query_value_by_cell_index(&backend, refinement_level.get() as u32, 0, &cell1)
            .expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 20);

        let bytes = query_value_by_cell_index(&backend, refinement_level.get() as u32, 0, &cell2)
            .expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 60);

        let _ = std::fs::remove_dir_all(&store_path);
    }
}
