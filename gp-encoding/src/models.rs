// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use clap::ValueEnum;
use geoplegma::types::DggrsUid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[allow(non_camel_case_types)]
#[serde(remote = "DggrsUid")]
pub enum DggrsUidDef {
    ISEA3HDGGRID,
    IGEO7,
    H3,
    IVEA3H,
    ISEA3HDGGAL,
    IVEA9R,
    ISEA9R,
    RTEA3H,
    RTEA9R,
    IVEA7H,
    IVEA7H_Z7,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// DGGS Reference System identifier
    #[serde(with = "DggrsUidDef")]
    pub dggrs: DggrsUid,

    /// Schema of the stored attributes.
    pub attributes: Vec<AttributeSchema>,

    /// Chunk size: number of cells per chunk along the linearized SFC axis.
    pub chunk_size: u64,

    /// Resolution levels stored in this dataset.
    pub levels: Vec<u32>,

    /// Compression method used for the cell attribute data.
    pub compression: Option<Compression>,
}

/// Supported compression configuration for Zarr chunk payloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(tag = "algorithm", rename_all = "snake_case")]
pub enum Compression {
    Gzip,
    Zstd,
}

/// Description of a single data attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSchema {
    /// Data type of the attribute values.
    pub dtype: DataType,

    /// Optional fill / no-data value.
    pub fill_value: Option<String>,
}

/// Supported data types for cell attribute values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Float32,
    Float64,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
}

impl DataType {
    pub const fn byte_size(&self) -> usize {
        match self {
            DataType::Int8 | DataType::UInt8 => 1,
            DataType::Int16 | DataType::UInt16 => 2,
            DataType::Float32 | DataType::Int32 | DataType::UInt32 => 4,
            DataType::Float64 | DataType::Int64 | DataType::UInt64 => 8,
        }
    }
}
