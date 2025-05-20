// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{
    models::common::{Position2D, PositionGeo},
    projections::constants::{ELIPSOID_MAJOR, ELIPSOID_MINOR},
};

use super::{layout::Layout, polyhedron::Polyhedron};

pub trait Projection {
    fn forward(
        &self,
        positions: Vec<PositionGeo>,
        polyhedron: Option<&dyn Polyhedron>,
        layout: &dyn Layout,
    ) -> Vec<Position2D>;
    fn inverse(&self) -> String;

    fn to_3d(lat: f64, lon: f64) -> [f64; 3] {
        let x = lat.cos() * lon.cos();
        let y = lat.cos() * lon.sin();
        let z = lat.sin();

        [x, y, z]
    }

    /// https://arxiv.org/pdf/2212.05818 (Karney, 2023)
    /// ** Convert authalic latitude to geodetic latitude (or vice-versa) **
    /// This process can also be done between geodetic latitude and other auxiliar latitudes.
    /// 1. Choose elipsoid and calculate the flattening
    /// 2. Evalute fourier coefficients (using the Horner method)
    /// 3. Apply Clenshaw summation
    fn lat_authalic_to_geodetic(latitude: f64, coef: &Vec<f64>) -> f64 {
        Self::apply_clenshaw_summation(latitude, coef)
    }

    fn lat_geodetic_to_authalic(latitude: f64, coef: &Vec<f64>) -> f64 {
        Self::apply_clenshaw_summation(latitude, coef)
    }

    // Used the Horner method
    // F(L×M)ηζ = C(L×M)ηζ · P(M)(n) where L = M = 6 => smallest matrix with accuracy
    // ex: c1*n + c2*n² + c3*n³ + ... cn*n^n
    fn fourier_coefficients(c: [f64; 21]) -> Vec<f64> {
        // Third flattening of the ellipsoid
        let n = (ELIPSOID_MAJOR - ELIPSOID_MINOR) / (ELIPSOID_MAJOR + ELIPSOID_MINOR);
        let mut coef: Vec<f64> = Vec::with_capacity(6);

        coef.push(
            c[0] * n
                + c[1] * n.powi(2)
                + c[2] * n.powi(3)
                + c[3] * n.powi(4)
                + c[4] * n.powi(5)
                + c[5] * n.powi(6),
        );

        coef.push(
            c[6] * n.powi(2)
                + c[7] * n.powi(3)
                + c[8] * n.powi(4)
                + c[9] * n.powi(5)
                + c[10] * n.powi(6),
        );

        coef.push(c[11] * n.powi(3) + c[12] * n.powi(4) + c[13] * n.powi(5) + c[14] * n.powi(6));

        coef.push(c[15] * n.powi(4) + c[16] * n.powi(5) + c[17] * n.powi(6));

        coef.push(c[18] * n.powi(5) + c[19] * n.powi(6));

        coef.push(c[20] * n.powi(6));

        coef
    }

    fn apply_clenshaw_summation(latitude: f64, coef: &Vec<f64>) -> f64 {
        // Clenshaw summation (1955) (order 6)
        let mut u0 = 0.0;
        let mut u1 = 0.0;
        let sin_zeta = latitude.sin();
        let cos_zeta = latitude.cos();
        let x = (cos_zeta - sin_zeta) * (cos_zeta + sin_zeta);

        let mut k = 6;
        while k > 0 {
            k -= 1;
            let t = 2.0 * x * u0 - u1 + coef[k];
            u1 = u0;
            u0 = t;
        } // Equation (33) (Karney, 2023)

        latitude + 2.0 * u0 * sin_zeta * cos_zeta
    }
}
