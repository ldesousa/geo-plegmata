// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::num::TryFromIntError;
use thiserror::Error;

/// Error type for zone-related logic in DGGAL-based adapters.
#[derive(Debug, Error)]
pub enum DggridError {
    #[error("Failed to convert edge count to u32 for zone ID '{zone_id}': {source}")]
    EdgeCountConversion {
        zone_id: String,
        #[source]
        source: TryFromIntError,
    },

    #[error("Invalid zone ID format: '{0}'")]
    InvalidZoneIdFormat(String),

    #[error("Invalid Z3 format: '{0}'")]
    InvalidZ3Format(String),

    #[error("Invalid Z7 format: '{0}'")]
    InvalidZ7Format(String),

    #[error("Missing required zone data")]
    MissingZoneData,

    // File I/O
    #[error("Failed to read file {path}")]
    FileRead {
        path: String,
        #[source]
        source: io::Error,
    },

    // Generic I/O passthrough (when you don't need the path)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    // When the format itself is broken (no underlying source)
    #[error("Malformed DGGRID content: {msg}")]
    Malformed { msg: String },
}
