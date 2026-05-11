// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod dggal;
pub mod dggrid;
pub mod factory;
pub mod h3o;
pub mod port;

use crate::error::dggal::DggalError;
use crate::error::dggrid::DggridError;
use crate::error::factory::FactoryError;
use crate::error::h3o::H3oError;
use crate::types::{RefinementLevel, RelativeDepth};
use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DggrsError {
    #[error("Factory error: {0}")]
    Factory(#[from] FactoryError),

    #[error("DGGAL error: {0}")]
    Dggal(#[from] DggalError),

    #[error("DGGRID error: {0}")]
    Dggrid(#[from] DggridError),

    #[error("H3o error: {0}")]
    H3o(#[from] H3oError),

    #[error("Depth must be non-negative, got {0}")]
    DepthBelowZero(i32),

    #[error("Relative depth must be non-negative, got {0}")]
    RelativeDepthBelowZero(i32),

    #[error("Unsupported tool/grid combination: {tool}, {grid}")]
    UnsupportedCombo { tool: String, grid: String },

    #[error("Requested depth {requested} exceeds maximum allowed {maximum} for grid '{grid_name}'")]
    RefinementLevelLimitReached {
        grid_name: String,
        requested: RefinementLevel,
        maximum: RefinementLevel,
    },

    #[error(
        "Requested relative depth {requested} exceeds maximum allowed {maximum} for grid '{grid_name}'"
    )]
    RelativeDepthLimitReached {
        grid_name: String,
        requested: RelativeDepth,
        maximum: RelativeDepth,
    },

    #[error(
        "Requested zone refinement level + relative depth {requested} exceeds maximum allowed {maximum} for grid '{grid_name}'"
    )]
    RefinementLevelPlusRelativeDepthLimitReached {
        grid_name: String,
        requested: RelativeDepth,
        maximum: RefinementLevel,
    },

    #[error("Depth too large to convert to u8: {0}")]
    RefinementLevelTooHigh(RefinementLevel),

    #[error("Relative depth too large to convert to u8: {0}")]
    RelativeDepthTooLarge(RelativeDepth),

    #[error("Unsupported ZoneId format '{0}'")]
    UnsupportedZoneIdFormat(String),

    #[error("Invalid hex ZoneId: '{0}'")]
    InvalidHexId(String),

    // Parsing primitives
    #[error("Float parse error: {0}")]
    Float(#[from] ParseFloatError),
}
