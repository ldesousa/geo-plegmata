use std::f64::consts::PI;

use crate::utils::math::to_rad;

// use super::constants::{AUTHALIC_EARTH_RADIUS, RR, SPHERICAL_DISTANCE, THETA};

/// pscp - Parallel Small Circle Projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687

pub struct Pscp {}



impl Pscp {
    fn new() {
        // /// Spherical distance 
        // let PD: f64 = to_rad(SPHERICAL_DISTANCE);
        // let angle_beta: f64 = to_rad(90.0 - THETA);
        // let vector_radius: f64 = RR * AUTHALIC_EARTH_RADIUS;

        // /// 1. Calculate b
        // let b = PI / 2.0 - PD;

        // /// 2. Calculate g
        // let g = f64::asin(f64::sin(PI / 2.0) * f64::sin(b) / f64::sin(angle_beta));

        // /// 3. Calculate phi and f
        // let phi = 2.0
        //     * f64::atan(
        //         f64::sin((g - b) / 2.0) * f64::tan(((PI / 2.0) - angle_beta) / 2.0)
        //             / f64::sin((g + b) / 2.0),
        //     );

        // let f = 2.0
        //     * f64::atan(
        //         f64::sin(((PI / 2.0) + angle_beta) / 2.0) * f64::tan((g - b) / 2.0)
        //             / f64::sin(((PI / 2.0) - angle_beta) / 2.0),
        //     );

        // /// 4. Calculate d
        // let BC = vector_radius * f64::tan(to_rad(SPHERICAL_DISTANCE));
        // let AB = BC / f64::cos(angle_beta);
        // let d = AB - f;

        // /// 5. Calculate d
        // let angle_upsilon = f64::acos(AB / ;BC)
        // let u = f64::sqrt(angle_upsilon - phi - d * f64::sin(b));
        // let v = f64::sqrt(angle_beta - angle_upsilon - PI / 2.0) - u;

        // /// Positioning
        // /// THIS PART I DONT KNOW
        // let angle_psi = 0.0;
        // /// 1. calculate angle Chi
        // let angle_chi = d - angle_psi;

        // let x = angle_chi;
        // let y = angle_psi - x;
    }
}
