// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use geoplegma::get;
use rayon::prelude::*;
use geoplegma::types::{RefinementLevel, RelativeDepth, ZoneId};
use indicatif::{ProgressBar, ProgressStyle};
use zarrs::array::codec::api::BytesToBytesCodecTraits;
use zarrs::array::codec::{GzipCodec, ZstdCodec};
use zarrs::array::{Array, ArrayBuilder, ArrayBytes, FillValue};
use zarrs::group::{Group, GroupBuilder};
use zarrs_filesystem::FilesystemStore;

use crate::common::ID_ONLY_CONFIG;
use crate::error::EncodingError;
use crate::models::{Compression, DataType, DatasetMetadata};
use crate::storage::{LevelHandle, StorageBackend};
use crate::value::{decode_value_to_f64, encode_value_from_f64, parse_fill_value_to_f64};
use serde_json::json;

/// Handle to a Zarr array that represents a single resolution level.
pub struct ZarrLevel {
    /// Resolution level index.
    pub level: u32,
    /// Number of cells at this level.
    pub num_cells: u64,
}

impl LevelHandle for ZarrLevel {}

/// A [`StorageBackend`] implementation that persists DGGS data in Zarr.
pub struct ZarrBackend {
    /// Root path of the Zarr store on disk.
    _root: PathBuf,
    /// The underlying filesystem store.
    store: Arc<FilesystemStore>,
    /// Dataset metadata.
    metadata: DatasetMetadata,
    /// Maps created resolution levels to num_cells.
    level_map: BTreeMap<u32, u64>,
}

impl ZarrBackend {
    fn codecs_for_compression(
        compression: Option<&Compression>,
    ) -> Result<Option<Vec<Arc<dyn BytesToBytesCodecTraits>>>, EncodingError> {
        let Some(config) = compression else {
            return Ok(None);
        };

        match config {
            Compression::Gzip => {
                let codec: Arc<dyn BytesToBytesCodecTraits> =
                    Arc::new(GzipCodec::new(6).map_err(|e| EncodingError::Zarr(e.to_string()))?);
                Ok(Some(vec![codec]))
            }
            Compression::Zstd => {
                let codec: Arc<dyn BytesToBytesCodecTraits> = Arc::new(ZstdCodec::new(3, false));
                Ok(Some(vec![codec]))
            }
        }
    }

    fn to_zarr_dtype_str(dt: &DataType) -> &'static str {
        match dt {
            DataType::Float32 => "float32",
            DataType::Float64 => "float64",
            DataType::Int8 => "int8",
            DataType::Int16 => "int16",
            DataType::Int32 => "int32",
            DataType::Int64 => "int64",
            DataType::UInt8 => "uint8",
            DataType::UInt16 => "uint16",
            DataType::UInt32 => "uint32",
            DataType::UInt64 => "uint64",
        }
    }

    fn level_path(level: u32, band: u32) -> String {
        format!("/level_{level}/band_{band}")
    }

    fn level_group_path(level: u32) -> String {
        format!("/level_{level}")
    }

    /// Open a Zarr array for a given resolution level.
    fn open_array(&self, level: u32, band: u32) -> Result<Array<FilesystemStore>, EncodingError> {
        let path = Self::level_path(level, band);
        let array = Array::open(self.store.clone(), &path).map_err(|e| {
            EncodingError::Zarr(format!(
                "failed to open array {path} in store {}: {e}",
                self._root.display()
            ))
        })?;
        Ok(array)
    }

    fn write_metadata(
        store: &Arc<FilesystemStore>,
        metadata: &DatasetMetadata,
    ) -> Result<(), EncodingError> {
        let attrs_json = serde_json::to_value(metadata)?;
        let mut attributes = serde_json::Map::new();
        attributes.insert("gp_encoding".to_string(), attrs_json);

        let group = GroupBuilder::new()
            .attributes(attributes)
            .build(store.clone(), "/")
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        group
            .store_metadata()
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        Ok(())
    }

    fn read_metadata(store: &Arc<FilesystemStore>) -> Result<DatasetMetadata, EncodingError> {
        let group =
            Group::open(store.clone(), "/").map_err(|e| EncodingError::Zarr(e.to_string()))?;

        let attrs = group.attributes();

        let meta_value = attrs.get("gp_encoding").ok_or_else(|| {
            EncodingError::Zarr("missing 'gp_encoding' attribute in root group".into())
        })?;

        let metadata: DatasetMetadata = serde_json::from_value(meta_value.clone())?;
        Ok(metadata)
    }

    fn write_level_chunk_ids(
        store: &Arc<FilesystemStore>,
        level: u32,
        chunk_level: u32,
        chunk_ids: &[String],
    ) -> Result<(), EncodingError> {
        let mut attributes = serde_json::Map::new();
        attributes.insert("chunk_level".to_string(), json!(chunk_level));
        attributes.insert("chunk_ids".to_string(), json!(chunk_ids));

        let group_path = Self::level_group_path(level);
        let group = GroupBuilder::new()
            .attributes(attributes)
            .build(store.clone(), &group_path)
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        group
            .store_metadata()
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        Ok(())
    }

    fn read_level_chunk_ids(
        store: &Arc<FilesystemStore>,
        level: u32,
    ) -> Result<(u32, Vec<String>), EncodingError> {
        let group_path = Self::level_group_path(level);
        let group = Group::open(store.clone(), &group_path)
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        let attrs = group.attributes();
        
        let level_val = attrs.get("chunk_level").ok_or_else(|| {
            EncodingError::Zarr(format!("missing 'chunk_level' attribute for level {level}"))
        })?;
        let chunk_level: u32 = serde_json::from_value(level_val.clone())?;

        let value = attrs.get("chunk_ids").ok_or_else(|| {
            EncodingError::Zarr(format!("missing 'chunk_ids' attribute for level {level}"))
        })?;

        let chunk_ids: Vec<String> = serde_json::from_value(value.clone())?;
        Ok((chunk_level, chunk_ids))
    }

    pub fn add_level_from_existing(
        &mut self,
        source_level: u32,
        target_level: u32,
    ) -> Result<(), EncodingError> {
        if source_level == target_level {
            return Err(EncodingError::Storage(
                "source and target levels must differ".into(),
            ));
        }

        if target_level > source_level {
            return Err(EncodingError::Storage(
                "target level must be lower than source level".into(),
            ));
        }

        if self.metadata.levels.contains(&target_level) {
            return Err(EncodingError::Storage(format!(
                "level {target_level} already exists in the store"
            )));
        }

        let chunk_size = self.metadata.chunk_size;
        if chunk_size == 0 {
            return Err(EncodingError::Storage(
                "dataset metadata chunk_size must be greater than zero".into(),
            ));
        }

        let source_level_i32 = i32::try_from(source_level)
            .map_err(|_| EncodingError::Storage(format!("level {source_level} cannot fit i32")))?;
        let target_level_i32 = i32::try_from(target_level)
            .map_err(|_| EncodingError::Storage(format!("level {target_level} cannot fit i32")))?;

        let (source_chunk_level_u32, source_chunk_ids) = Self::read_level_chunk_ids(&self.store, source_level)?;
        let source_chunk_level = RefinementLevel::new(source_chunk_level_u32 as i32)?;
        
        let grid = get(self.metadata.dggrs)
            .map_err(|e| EncodingError::Grid(format!("failed to resolve DGGS: {e}")))?;

        let min_chunk_level = grid.min_refinement_level().map_err(|e| EncodingError::Grid(e.to_string()))?;
        let max_relative_depth_allowed = grid.max_relative_depth().map_err(|e| EncodingError::Grid(e.to_string()))?;
        let aperture = u64::from(self.metadata.dggrs.spec().aperture);
        let data_type_size_bytes = self.metadata.attributes.first().map(|a| a.dtype.byte_size()).unwrap_or(1);

        let (target_chunk_level, chunk_size) = crate::convert::choose_best_chunk_level_and_size(
            RefinementLevel::new(target_level_i32)?,
            min_chunk_level,
            max_relative_depth_allowed,
            aperture,
            data_type_size_bytes,
        )?;

        if source_chunk_ids.is_empty() {
            return Err(EncodingError::Storage(
                "source level has no chunk IDs".into(),
            ));
        }

        let chunk_level_steps = source_chunk_level.get() - target_chunk_level.get();

        let map_progress = ProgressBar::new(source_chunk_ids.len() as u64);
        let map_style = ProgressStyle::with_template(
            "mapping chunks [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)",
        )
        .map_err(|e| EncodingError::Storage(format!("invalid progress bar template: {e}")))?
        .progress_chars("=> ");
        map_progress.set_style(map_style);

        // Map source chunks to target chunks in parallel
        let mapped_chunks: Vec<(String, (usize, String))> = source_chunk_ids
            .par_iter()
            .enumerate()
            .map(|(source_chunk_index, source_chunk_id)| {
                let mut zone_id = ZoneId::from_str(source_chunk_id.as_str()).map_err(|e| {
                    EncodingError::Storage(format!("invalid chunk id '{source_chunk_id}': {e}"))
                })?;

                for _ in 0..chunk_level_steps {
                    let parent = grid
                        .primary_parent_from_zone(zone_id, Some(ID_ONLY_CONFIG))
                        .map_err(|e| EncodingError::Grid(e.to_string()))?;
                    let parent_zone = parent.zones.first().ok_or_else(|| {
                        EncodingError::Grid("primary_parent_from_zone returned no zones".into())
                    })?;
                    zone_id = parent_zone.id.clone();
                }

                map_progress.inc(1);
                Ok((zone_id.to_string(), (source_chunk_index, source_chunk_id.clone())))
            })
            .collect::<Result<Vec<_>, EncodingError>>()?;
        map_progress.finish_with_message("mapping chunks [done]");

        let mut target_to_sources: HashMap<String, Vec<(usize, String)>> = HashMap::new();
        for (target_chunk_id, source_info) in mapped_chunks {
            target_to_sources.entry(target_chunk_id).or_default().push(source_info);
        }

        let mut target_chunk_ids: Vec<String> = target_to_sources.keys().cloned().collect();
        target_chunk_ids.sort();

        if target_chunk_ids.is_empty() {
            return Err(EncodingError::Storage(
                "target level has no chunk IDs".into(),
            ));
        }

        let relative_depth = RelativeDepth::new(target_level_i32 - target_chunk_level.get())?;
        let band_count = self.band_count() as usize;

        let source_relative_depth =
            RelativeDepth::new(source_level_i32 - source_chunk_level.get())?;
        let level_steps = source_level_i32 - target_level_i32;
        let fill_values: Vec<f64> = self
            .metadata
            .attributes
            .iter()
            .map(|attr| parse_fill_value_to_f64(&attr.dtype, &attr.fill_value))
            .collect::<Result<_, EncodingError>>()?;

        self.set_level_chunk_ids(target_level, target_chunk_level.get() as u32, target_chunk_ids.clone())?;

        let encoded_num_cells = (target_chunk_ids.len() as u64)
            .checked_mul(chunk_size)
            .ok_or_else(|| EncodingError::Storage("encoded cell count overflow".into()))?;

        for band in 0..band_count {
            self.create_level(target_level, band as u32, encoded_num_cells, chunk_size)?;
        }

        let fill_bytes: Vec<Vec<u8>> = self
            .metadata
            .attributes
            .iter()
            .map(|attr| {
                let fill_value = parse_fill_value_to_f64(&attr.dtype, &attr.fill_value)?;
                encode_value_from_f64(&attr.dtype, fill_value)
            })
            .collect::<Result<_, EncodingError>>()?;

        let process_progress = ProgressBar::new(target_chunk_ids.len() as u64);
        let process_style = ProgressStyle::with_template(
            "processing and writing target chunks [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)",
        )
        .map_err(|e| EncodingError::Storage(format!("invalid progress bar template: {e}")))?
        .progress_chars("=> ");
        process_progress.set_style(process_style);

        target_chunk_ids
            .par_iter()
            .enumerate()
            .try_for_each(|(target_chunk_index, target_chunk_id)| {
                let chunk_zone_id = ZoneId::from_str(target_chunk_id).map_err(|e| {
                    EncodingError::Storage(format!("invalid chunk id '{target_chunk_id}': {e}"))
                })?;

                let children = grid
                    .zones_from_parent(relative_depth, chunk_zone_id, Some(ID_ONLY_CONFIG))
                    .map_err(|e| EncodingError::Grid(e.to_string()))?;

                if children.zones.len() > chunk_size as usize {
                    return Err(EncodingError::Storage(format!(
                        "chunk {target_chunk_id} has {} children but chunk_size is {}",
                        children.zones.len(),
                        chunk_size
                    )));
                }

                let mut cell_index_map: HashMap<ZoneId, usize> = HashMap::with_capacity(children.zones.len());
                for (cell_index, zone) in children.zones.iter().enumerate() {
                    cell_index_map.insert(zone.id.clone(), cell_index);
                }

                let mut band_accumulators = Vec::with_capacity(band_count);
                for _ in 0..band_count {
                    band_accumulators.push(BandAccumulator::new(chunk_size as usize));
                }

                if let Some(src_list) = target_to_sources.get(target_chunk_id) {
                    for &(source_chunk_index, ref source_chunk_id) in src_list {
                        let src_chunk_zone_id = ZoneId::from_str(source_chunk_id.as_str()).map_err(|e| {
                            EncodingError::Storage(format!("invalid chunk id '{source_chunk_id}': {e}"))
                        })?;

                        let src_children = grid
                            .zones_from_parent(source_relative_depth, src_chunk_zone_id, Some(ID_ONLY_CONFIG))
                            .map_err(|e| EncodingError::Grid(e.to_string()))?;

                        let chunks_for_bands: Vec<Vec<u8>> = (0..band_count)
                            .map(|band| self.read_chunk(source_level, band as u32, source_chunk_index as u64))
                            .collect::<Result<Vec<_>, _>>()?;

                        for (in_chunk_index, zone) in src_children.zones.iter().enumerate() {
                            let mut target_zone_id = zone.id.clone();
                            for _ in 0..level_steps {
                                let parent = grid
                                    .primary_parent_from_zone(target_zone_id, Some(ID_ONLY_CONFIG))
                                    .map_err(|e| EncodingError::Grid(e.to_string()))?;
                                let parent_zone = parent.zones.first().ok_or_else(|| {
                                    EncodingError::Grid("primary_parent_from_zone returned no zones".into())
                                })?;
                                target_zone_id = parent_zone.id.clone();
                            }

                            if let Some(&target_cell_index) = cell_index_map.get(&target_zone_id) {
                                for band in 0..band_count {
                                    let dtype = &self.metadata.attributes[band].dtype;
                                    let value_size = dtype.byte_size();
                                    let chunk = &chunks_for_bands[band];
                                    let start = in_chunk_index * value_size;
                                    let end = start + value_size;
                                    if chunk.len() < end {
                                        return Err(EncodingError::Storage(format!(
                                            "chunk {source_chunk_index} at level {source_level} is too small for child index {in_chunk_index}"
                                        )));
                                    }

                                    let value = decode_value_to_f64(dtype, &chunk[start..end])?;
                                    if value == fill_values[band]
                                        || (value.is_nan() && fill_values[band].is_nan())
                                    {
                                        continue;
                                    }

                                    band_accumulators[band].record(target_cell_index, value);
                                }
                            }
                        }
                    }
                }

                for band in 0..band_count {
                    let dtype = &self.metadata.attributes[band].dtype;
                    let mut bytes = Vec::with_capacity(chunk_size as usize * dtype.byte_size());

                    for cell_index in 0..chunk_size as usize {
                        let value_bytes = if let Some(value) = band_accumulators[band].average(cell_index) {
                            encode_value_from_f64(dtype, value)?
                        } else {
                            fill_bytes[band].clone()
                        };
                        bytes.extend_from_slice(&value_bytes);
                    }

                    self.write_chunk(target_level, band as u32, target_chunk_index as u64, &bytes)?;
                }

                process_progress.inc(1);
                Ok(())
            })?;
        process_progress.finish_with_message("processing and writing target chunks [done]");

        self.metadata.levels.push(target_level);
        self.metadata.levels.sort_unstable();

        Self::write_metadata(&self.store, &self.metadata)?;

        Ok(())
    }
}

struct BandAccumulator {
    sums: Vec<f64>,
    counts: Vec<u64>,
}

impl BandAccumulator {
    fn new(size: usize) -> Self {
        Self {
            sums: vec![0.0; size],
            counts: vec![0; size],
        }
    }

    fn record(&mut self, index: usize, value: f64) {
        self.sums[index] += value;
        self.counts[index] += 1;
    }

    fn average(&self, index: usize) -> Option<f64> {
        let count = self.counts[index];
        if count == 0 {
            None
        } else {
            Some(self.sums[index] / count as f64)
        }
    }
}

impl StorageBackend for ZarrBackend {
    type Level = ZarrLevel;

    fn create(path: &Path, metadata: DatasetMetadata) -> Result<Self, EncodingError> {
        std::fs::create_dir_all(path)?;

        let store =
            Arc::new(FilesystemStore::new(path).map_err(|e| EncodingError::Zarr(e.to_string()))?);

        Self::write_metadata(&store, &metadata)?;

        Ok(Self {
            _root: path.to_path_buf(),
            store,
            metadata,
            level_map: BTreeMap::new(),
        })
    }

    fn open(path: &Path) -> Result<Self, EncodingError> {
        if !path.exists() {
            return Err(EncodingError::Storage(format!(
                "store path does not exist: {}",
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(EncodingError::Storage(format!(
                "store path is not a directory: {}",
                path.display()
            )));
        }
        let store =
            Arc::new(FilesystemStore::new(path).map_err(|e| EncodingError::Zarr(e.to_string()))?);

        let metadata = Self::read_metadata(&store)?;

        // Discover existing level arrays.
        let mut level_map = BTreeMap::new();
        for &lvl in &metadata.levels {
            for band in 0..metadata.attributes.len() as u32 {
                let array_path = Self::level_path(lvl, band);
                if let Ok(array) = Array::open(store.clone(), &array_path) {
                    let shape = array.shape();
                    let num_cells = shape.first().copied().unwrap_or(0);
                    level_map.insert(lvl, num_cells);
                }
            }
        }

        Ok(Self {
            _root: path.to_path_buf(),
            store,
            metadata,
            level_map,
        })
    }

    fn metadata(&self) -> &DatasetMetadata {
        &self.metadata
    }

    fn create_level(
        &mut self,
        level: u32,
        band: u32,
        num_cells: u64,
        chunk_size: u64,
    ) -> Result<ZarrLevel, EncodingError> {
        let path = Self::level_path(level, band);
        let attribute = self.metadata.attributes.get(band as usize).ok_or_else(|| {
            EncodingError::Storage(format!("band {band} is not present in dataset metadata"))
        })?;
        let dtype_str = Self::to_zarr_dtype_str(&attribute.dtype);
        let fill_value = parse_fill_value_to_f64(&attribute.dtype, &attribute.fill_value)?;
        let fill_bytes = encode_value_from_f64(&attribute.dtype, fill_value)?;

        let mut array_builder = ArrayBuilder::new(
            vec![num_cells],
            vec![chunk_size],
            dtype_str,
            FillValue::new(fill_bytes),
        );

        if let Some(codecs) = Self::codecs_for_compression(self.metadata.compression.as_ref())? {
            array_builder.bytes_to_bytes_codecs(codecs);
        }

        let array = array_builder
            .build(self.store.clone(), &path)
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        array
            .store_metadata()
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        self.level_map.insert(level, num_cells);

        Ok(ZarrLevel { level, num_cells })
    }

    fn levels(&self) -> Vec<u32> {
        self.level_map.keys().copied().collect()
    }

    fn band_count(&self) -> u32 {
        self.metadata.attributes.len() as u32
    }

    fn write_chunk(
        &self,
        level: u32,
        band: u32,
        chunk_index: u64,
        data: &[u8],
    ) -> Result<(), EncodingError> {
        let array = self.open_array(level, band)?;

        let array_bytes = ArrayBytes::new_flen(data.to_vec());
        array
            .store_chunk(&[chunk_index], array_bytes)
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        Ok(())
    }

    fn read_chunk(
        &self,
        level: u32,
        band: u32,
        chunk_index: u64,
    ) -> Result<Vec<u8>, EncodingError> {
        let array = self.open_array(level, band)?;

        let bytes: ArrayBytes<'static> = array
            .retrieve_chunk(&[chunk_index])
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        let fixed = bytes
            .into_fixed()
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        Ok(fixed.into_owned())
    }

    fn num_chunks(&self, level: u32) -> Result<u64, EncodingError> {
        let num_cells = *self
            .level_map
            .get(&level)
            .ok_or_else(|| EncodingError::Zarr(format!("level {level} not found")))?;
        
        // Read chunk size from the first band's array metadata
        let array = self.open_array(level, 0)?;
        let chunk_size = array.chunk_shape(&[0])
            .map_err(|e| EncodingError::Zarr(e.to_string()))?[0]
            .get();
        
        Ok(num_cells.div_ceil(chunk_size))
    }

    fn chunk_ids_for_level(&self, level: u32) -> Result<(u32, Vec<String>), EncodingError> {
        Self::read_level_chunk_ids(&self.store, level)
    }

    fn set_level_chunk_ids(
        &mut self,
        level: u32,
        chunk_level: u32,
        chunk_ids: Vec<String>,
    ) -> Result<(), EncodingError> {
        Self::write_level_chunk_ids(&self.store, level, chunk_level, &chunk_ids)
    }
}
