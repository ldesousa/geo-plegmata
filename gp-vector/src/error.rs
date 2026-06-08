// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geoplegma::types::Point;

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Geoplegma error: {0}")]
    Geoplegma(#[from] geoplegma::error::DggrsError),

    #[error("Factory error: {0}")]
    Factory(#[from] geoplegma::error::factory::FactoryError),

    #[error("Invalid coordinate length (expected at least 2, got {0})")]
    InvalidCoordinateLength(usize),

    #[error("Invalid coordinate format (expected floats / arrays of floats)")]
    InvalidCoordinateFormat,

    #[error("No zone found for point {0:?}")]
    NoZoneFound(Point),
}
