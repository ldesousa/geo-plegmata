// Copyright 2025 contributors to the GeoPlegma project.
//
// Authored by Sunayana Ghosh (Independent Researcher, sunayanag@gmail.com)
// Licensed under the Apache License, Version 2.0 <LICENCE-APACHE-2.0 or 
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENCE-MIT 
// or http://opensource.org/licenses/MIT>, at your discretion. This file may not 
// be copied, modified or distributed except according to those terms. 

pub mod authalic;
pub mod auxiliary_latitude;

pub use authalic::{AuthalicCoord, AuthalicSphere};


/// A reference ellipsoid, described by the parameters needed to convert
/// geodetic coordinates onto an equal-area (authalic) sphere.
pub trait Ellipsoid {
    /// Third flattening `n = (a - b) / (a + b)`.
    fn third_flattening(&self) -> f64;

    /// Radius of the sphere with the same surface area as this ellipsoid.
    fn authalic_radius(&self) -> f64;

    /// Semi-major axis `a` (equatorial radius), in meters.
    fn major_axis(&self) -> f64;

    /// Eccentricity squared `e² = (a² - b²) / a²`.
    fn eccentricity_squared(&self) -> f64;
}