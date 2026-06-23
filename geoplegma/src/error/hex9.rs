// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use thiserror::Error;

/// Error type for the libhex9-backed (Hex9 / H9) adapter.
#[derive(Debug, Error)]
pub enum Hex9Error {
    #[error("libhex9 call '{call}' failed (rc {rc})")]
    Ffi { call: &'static str, rc: i64 },

    #[error("invalid Hex9 ZoneId '{0}' — expected a canonical label like \"43527.4\"")]
    InvalidZoneId(String),

    #[error("Hex9 ZoneId must be a textual label (StrId/HexId), got integer id '{0}'")]
    UnsupportedZoneId(u64),

    #[error("Hex9 grid enumeration failed: {0}")]
    Grid(String),

    #[error("a Hex9 zone at layer 0 has no parent")]
    NoParent,

    #[error("Hex9 label contained an interior NUL byte: '{0}'")]
    NulInLabel(String),
}
