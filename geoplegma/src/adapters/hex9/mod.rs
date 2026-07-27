// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

//! The libhex9-backed Hex9 (H9) DGGRS adapter — an aperture-9 hexagonal grid
//! with self-contained 16-byte UUID addresses, bound via the `hex9-sys` crate.

pub mod common;
pub mod context;
pub mod grids;
