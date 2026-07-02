// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("GDAL error: {0}")]
    Gdal(#[from] gdal::errors::GdalError),

    #[error("DGGRS error: {0}")]
    Dggrs(#[from] geoplegma::error::DggrsError),

    #[error("DGGRS Fabric error: {0}")]
    DggrsFabric(#[from] geoplegma::error::factory::FactoryError),

    #[error("Dataset error: {0}")]
    Dataset(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Zarr backend error: {0}")]
    Zarr(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Grid error: {0}")]
    Grid(String),
}
