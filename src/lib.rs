// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc = include_str!("../README.md")]
pub mod adapters;
pub mod dggrs;
pub mod factory;
pub mod models;
pub mod ports;

/// This is the only re-export that is needed.
pub use factory::dggrs_factory::get;
