// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::Path;

use crate::error::EncodingError;
use crate::models::DatasetMetadata;

pub trait LevelHandle: Send + Sync {}
pub trait StorageBackend: Sized + Send + Sync {
    type Level: LevelHandle;

    fn create(path: &Path, metadata: DatasetMetadata) -> Result<Self, EncodingError>;

    fn open(path: &Path) -> Result<Self, EncodingError>;

    fn metadata(&self) -> &DatasetMetadata;

    fn create_level(
        &mut self,
        level: u32,
        band: u32,
        num_cells: u64,
        chunk_size: u64,
    ) -> Result<Self::Level, EncodingError>;

    fn levels(&self) -> Vec<u32>;
    fn band_count(&self) -> u32;

    fn write_chunk(
        &self,
        level: u32,
        band: u32,
        chunk_index: u64,
        data: &[u8],
    ) -> Result<(), EncodingError>;

    fn read_chunk(&self, level: u32, band: u32, chunk_index: u64)
    -> Result<Vec<u8>, EncodingError>;

    fn num_chunks(&self, level: u32) -> Result<u64, EncodingError>;

    fn chunk_ids_for_level(&self, level: u32) -> Result<(u32, Vec<String>), EncodingError>;

    fn set_level_chunk_ids(
        &mut self,
        level: u32,
        chunk_level: u32,
        chunk_ids: Vec<String>,
    ) -> Result<(), EncodingError>;
}

