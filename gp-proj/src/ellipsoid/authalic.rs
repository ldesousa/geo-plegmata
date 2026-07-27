// Copyright 2025 contributors to the GeoPlegma project.
//
// Authored by Sunayana Ghosh (Independent Researcher, sunayanag@gmail.com)
// Licensed under the Apache License, Version 2.0 <LICENCE-APACHE-2.0 or 
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENCE-MIT 
// or http://opensource.org/licenses/MIT>, at your discretion. This file may not 
// be copied, modified or distributed except according to those terms.

use geo::Point;

use crate::constants::KarneyCoefficients;
use crate::ellipsoid::auxiliary_latitude::{apply_clenshaw_summation, fourier_coefficients};
use crate::ellipsoid::Ellipsoid;

/// A point already expressed on the authalic sphere: longitude/latitude in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AuthalicCoord {
    pub lon: f64,
    pub lat: f64,
}

/// Converts geodetic coordinates for a given [`Ellipsoid`] onto its authalic sphere.
/// 
/// Built once per ellipsoid (the Clenshaw coefficients are computed once), 
/// then reused to convert any number of points via [`AuthalicSphere::convert`].
pub struct AuthalicSphere {
    coefficients: Vec<f64>,
    radius: f64,
}

impl AuthalicSphere {
    pub fn from_ellipsoid(ellipsoid: &(impl Ellipsoid + ?Sized)) -> Self {
        let coefficients = fourier_coefficients(
            KarneyCoefficients::GEODETIC_TO_AUTHALIC,
            ellipsoid.third_flattening(),
        );

        Self {
            coefficients,
            radius: ellipsoid.authalic_radius(),
        }
    }


    /// Convert a geodetic point (lon/lat in degrees) to authalic lon/lat (radians).
    pub fn convert(&self, point: Point) -> AuthalicCoord {
        AuthalicCoord {
            lon: point.x().to_radians(),
            lat: apply_clenshaw_summation(point.y().to_radians(), &self.coefficients),
        }
    }

    /// Radius (meters) of the sphere this converter projects onto.
    pub fn radius(&self) -> f64 {
        self.radius
    }
}