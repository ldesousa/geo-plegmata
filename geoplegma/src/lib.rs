// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc = include_str!("../../README.md")]
pub mod adapters;
pub mod api;
pub mod constants;
pub mod error;
pub mod factory;
pub mod types;

pub use api::DggrsApiConfig as config;
/// This is the only re-export that is needed.
pub use factory::{get, registry};
