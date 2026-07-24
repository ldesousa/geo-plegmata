// Copyright 2025 contributors to the GeoPlegma project.
//
// Authored by Sunayana Ghosh (Independent Researcher, sunayanag@gmail.com)
// Licensed under the Apache License, Version 2.0 <LICENCE-APACHE-2.0 or 
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENCE-MIT 
// or http://opensource.org/licenses/MIT>, at your discretion. This file may not 
// be copied, modified or distributed except according to those terms. 


//! Auxiliary latitude conversions (Karney, 2023): https://arxiv.org/pdf/2212.05818
//! 
//! Ellipsoid-agnostic building blocks for converting between geodetic latitude
//! and other auxiliary latitudes (e.g. authalic). The ellipsoid only enters 
//! through the third flattening `n` passed into `fourier_coefficients`.


/// Evaluate the Fourier coefficients (Horner method) for a given third flattening `n`.
/// 
///  F(L×M)ηζ = C(L×M)ηζ · P(M)(n) where L = M = 6.
pub fn fourier_coefficients(c: [f64; 21], n: f64) -> Vec<f64> {
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


/// Apply Clenshaw summation (1955, order 6) - equation (33) in Karney (2023).
pub fn apply_clenshaw_summation(latitude: f64, coef: &[f64]) -> f64 {
    let mut u0 = 0.0;
    let mut u1 = 0.0;
    let sin_zeta = latitude.sin();
    let cos_zeta = latitude.cos();
    let x = (cos_zeta - sin_zeta) * (cos_zeta + sin_zeta); // cos(2*latitude)

    let mut k = 6;
    while k > 0 {
        k -= 1;
        let t = 2.0 * x * u0 - u1 + coef[k];
        u1 = u0;
        u0 = t;
    }
    latitude + 2.0 * u0 * sin_zeta * cos_zeta
}