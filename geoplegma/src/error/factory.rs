// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::types::DggrsUid;
use std::fmt;
use thiserror::Error;

/// Error type for instantiating DggrsPort adapters via the factory.
#[derive(Debug, Error)]
pub enum FactoryError {
    #[error("Invalid DGGRS UID error: {0}")]
    InvalidDggrsUidError(#[from] DggrsUidError),
}

#[derive(Debug)]
pub enum DggrsUidError {
    /// used at the *factory* level when a valid enum has no adapter
    Unsupported { id: DggrsUid },

    /// (optional) only if you parse strings somewhere else
    Unknown {
        input: String,
        candidates: Vec<DggrsUid>,
    },
}

impl fmt::Display for DggrsUidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DggrsUidError::Unsupported { id } => {
                write!(f, "no adapter registered for DGGRS id '{}'", id)
            }

            DggrsUidError::Unknown { input, candidates } => {
                let list = candidates
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "unknown DGGRS id '{}'. Try one of: {}", input, list)
            }
        }
    }
}

impl std::error::Error for DggrsUidError {}
