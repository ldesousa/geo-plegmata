// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

pub mod icosahedron;
pub mod geometry;
pub mod polyhedron;
pub mod spherical_geometry;

// Re-export the main types
pub use polyhedron::Polyhedron;
pub use geometry::{Face, ArcLengths, Orientation};