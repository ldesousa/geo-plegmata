use crate::{
    models::common::{Position2D, PositionGeo},
    projections::constants::{ELIPSOID_MAJOR, ELIPSOID_MINOR},
    utils::math::{cos, pow, sin},
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
        let x = cos(lat) * cos(lon);
        let y = cos(lat) * sin(lon);
        let z = sin(lat);

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
    fn compute_fourier_coefficients(c: [f64; 21]) -> Vec<f64> {
        // Third flattening of the ellipsoid
        let n = (ELIPSOID_MAJOR - ELIPSOID_MINOR) / (ELIPSOID_MAJOR + ELIPSOID_MINOR);
        let mut coef: Vec<f64> = Vec::with_capacity(6);

        coef.push(
            c[0] * n
                + c[1] * pow(n, 2)
                + c[2] * pow(n, 3)
                + c[3] * pow(n, 4)
                + c[4] * pow(n, 5)
                + c[5] * pow(n, 6),
        );

        coef.push(
            c[6] * pow(n, 2)
                + c[7] * pow(n, 3)
                + c[8] * pow(n, 4)
                + c[9] * pow(n, 5)
                + c[10] * pow(n, 6),
        );

        coef.push(c[11] * pow(n, 3) + c[12] * pow(n, 4) + c[13] * pow(n, 5) + c[14] * pow(n, 6));

        coef.push(c[15] * pow(n, 4) + c[16] * pow(n, 5) + c[17] * pow(n, 6));

        coef.push(c[18] * pow(n, 5) + c[19] * pow(n, 6));

        coef.push(c[20] * pow(n, 6));

        coef
    }

    fn apply_clenshaw_summation(latitude: f64, coef: &Vec<f64>) -> f64 {
        // Clenshaw summation (1955) (order 6)
        let mut u0 = 0.0;
        let mut u1 = 0.0;
        let sin_zeta = sin(latitude);
        let cos_zeta = cos(latitude);
        let x = (cos_zeta - sin_zeta) * (cos_zeta + sin_zeta);

        let mut k = 6;
        while k > 0 {
            k -= 1;
            let t = 2.0 * x * u0 - u1 + coef[k];
            u1 = u0;
            u0 = t;
        } // (33)

        latitude + 2.0 * u0 * sin_zeta * cos_zeta
    }
}
