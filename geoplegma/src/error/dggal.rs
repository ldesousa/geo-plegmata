// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::num::TryFromIntError;
use thiserror::Error;

/// Error type for zone-related logic in DGGAL-based adapters.
#[derive(Debug, Error)]
pub enum DggalError {
    #[error("Failed to convert edge count to u32 for zone ID '{zone_id}': {source}")]
    EdgeCountConversion {
        zone_id: String,
        #[source]
        source: TryFromIntError,
    },

    #[error("Invalid zone ID format: '{0}'")]
    InvalidZoneIdFormat(String),

    #[error("Missing required zone data")]
    MissingZoneData,

    #[error("Unknown Grid: {grid_name}")]
    UnknownGrid { grid_name: String },

    #[error("Failed to acquire global lock")]
    LockFailure,

    #[error("Failed to convert max depth to u8 for grid '{grid_name}': {source}")]
    DepthConversion {
        grid_name: String,
        #[source]
        source: TryFromIntError,
    },
    #[error("Invalid DGGAL ZoneId, checked with getZoneArea() resulted in inf")]
    InvalidDggalZoneId,
}
