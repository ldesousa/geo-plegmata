// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

mod common;
pub mod error;
pub mod convert;
pub mod vector_convert;
pub mod models;
pub mod query;
pub mod stats;
pub mod storage;
pub mod value;
pub mod zarr;

pub use geoplegma::api::DggrsApi;
pub use geoplegma::types::{BoundingBox, RefinementLevel, RelativeDepth, Zone, ZoneId, Zones};

pub use convert::{compute_source_report, convert_to_backend, convert_dggrs_store_to_backend};
pub use vector_convert::{convert_vector_file_to_json, convert_geojson_in_place};
pub use models::{AttributeSchema, Compression, DataType, DatasetMetadata};
pub use query::{
    VisualizationCell, export_level_as_visualization_json, query_value_by_cell_index,
    query_value_for_point, write_level_as_visualization_json,
};
pub use storage::StorageBackend;
pub use value::{decode_value_to_json, format_value};
pub use zarr::ZarrBackend;
