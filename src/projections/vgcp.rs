use std::f64::consts::PI;

use crate::{
    geometries::polyhedron::{consts::{GOLDEN_RATIO, SPHERICAL_DISTANCE}, ArcLengths, Icosahedron},
    models::common::Position,
    utils::math::{cos, pow, sin, tan, to_rad},
};

// use super::constants::{AUTHALIC_EARTH_RADIUS, GOLDEN_RATIO_ICOSAHEDRON, RR};

/// pscp - Parallel Small Circle Projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687

pub struct Vgcp {}

// struct Triangle {
//     a: [f64; 2],
//     b: [f64; 2],
//     c: [f64; 2],
// }

// pub struct UnitVectors {
//     a: [f64; 3],
//     b: [f64; 3],
//     c: [f64; 3],
// }

// impl Triangle {

// }

// impl UnitVectors {
//     fn new(mut self) {
//         let aux = 1.0 / f64::sqrt(1.0 + pow(GOLDEN_RATIO, 2));
//         let aux1 =
//             pow(GOLDEN_RATIO, 2) / f64::sqrt(1.0 + pow(GOLDEN_RATIO, 2));

//         self.a = vec![0.0, aux, aux1]
//             .try_into()
//             .expect("Expected exactly 3 elements");
//         self.b = vec![aux, aux1, 0.0]
//             .try_into()
//             .expect("Expected exactly 3 elements");
//         self.c = vec![aux1, 0.0, aux]
//             .try_into()
//             .expect("Expected exactly 3 elements");
//     }
// }

impl Vgcp {
    pub fn new(positions: Position) -> Icosahedron {
        let ico = Icosahedron::new();
        let ArcLengths { ab, bp, ap, bc, ac, cp } = ico.triangle_arcs;
        let arc_p = SPHERICAL_DISTANCE;
        // let uvs = ico.triangle_uvs;
        // /// Arc lengths
        // let x: f64 = bp;
        // let ab: f64 = 0.0;
        // let pa: f64 = 0.0;
        
        // angle ρ
        let rho: f64 = f64::acos(cos(arc_p) - cos(ab) * cos(arc_p)) / (sin(ab) * sin(arc_p));

        // /// @TODO: ADD TO CONSTANTS OF ICOSAHEDRON
        let angle_beta: f64 = to_rad(72.0);
        let angle_phi: f64 = to_rad(72.0);
        let angle_bac: f64 = PI/2.0;

        // /// 1. Calculate delta (δ)
        let delta = f64::acos(f64::sin(rho) * f64::cos(ab));

        // /// 2. Calculate u
        let uv = (angle_beta + angle_phi - rho - delta) / (angle_beta + angle_phi - PI / 2.0);

        let cosXpY;
        if rho <= 0.1 {
            cosXpY = cos(ab);
        } else {
            cosXpY = 1.0 / (tan(rho) * tan(delta))
        }

        let xy = f64::sqrt((1.0 - cos(arc_p)) / (1.0 - cosXpY))

        
        let pdi_x = tri_coords.c[0] + (tri_coords.a[0] - tri_coords.c[0]) * xy;
        let pdi_y = tri_coords.c[1] + (tri_coords.a[1] - tri_coords.c[1]) * xy;

        let pdi_x = pdi_x + (pdi_x - tri_coords.b[0]) * xy;
        let pdi_y = pdi_y + (pdi_y - tri_coords.b[1]) * xy;

        // [pdi_x, pdi_y]
        // // let x = distance_b;
        // // let y = f64::acos(f64::cos(rho) / f64::tan(delta) - x);

        // let prime_y = prime_x * (1.0 - f64::sqrt((1.0 - f64::cos(x)) / (1.0 - f64::cos(x + y))));
        // let x = ;
    }
}
