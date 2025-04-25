use std::f64::consts::{E, PI};

use crate::{
    // shape::icosahedron::{consts::{GOLDEN_RATIO, SPHERICAL_DISTANCE}, ArcLengths, Icosahedron},
    models::common::Position,
    utils::math::{cos, pow, sin, tan, to_rad},
};

// use super::constants::{AUTHALIC_EARTH_RADIUS, GOLDEN_RATIO_ICOSAHEDRON, RR};

/// vgcp - Vertex Great Circle Projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub trait Vgcp {
    fn new(positions: Position) -> [f64; 2] {
        // let ico = Icosahedron::new();
        // let ArcLengths { ab, bp, ap, bc, ac, cp } = ico.triangle_arcs;
        // let arc_p = SPHERICAL_DISTANCE;
        // let v2d = ico.triangle_vertexes;
        // // let uvs = ico.triangle_uvs;
        
        // // angle ρ
        // let rho: f64 = f64::acos(cos(arc_p) - cos(ab) * cos(arc_p)) / (sin(ab) * sin(arc_p));

        // // /// @TODO: ADD TO CONSTANTS OF ICOSAHEDRON
        // let angle_beta: f64 = to_rad(36.0);
        // let angle_phi: f64 = to_rad(60.0);
        // let angle_bac: f64 = PI/2.0;

        // // /// 1. Calculate delta (δ)
        // let delta = f64::acos(f64::sin(rho) * f64::cos(ab));

        // // /// 2. Calculate u
        // let uv = (angle_beta + angle_phi - rho - delta) / (angle_beta + angle_phi - PI / 2.0);

        // let cosXpY;
        // if rho <= pow(E, -9) {
        //     cosXpY = cos(ab);
        // } else {
        //     cosXpY = 1.0 / (tan(rho) * tan(delta))
        // }

        // let xy = f64::sqrt((1.0 - cos(arc_p)) / (1.0 - cosXpY));

        
        // let pdi_x = v2d.c[0] + (v2d.a[0] - v2d.c[0]) * uv;
        // let pdi_y = v2d.c[1] + (v2d.a[1] - v2d.c[1]) * uv;

        // let pdi_x = pdi_x + (pdi_x - v2d.b[0]) * xy;
        // let pdi_y = pdi_y + (pdi_y - v2d.b[1]) * xy;

        // [pdi_x, pdi_y]
        // // // let x = distance_b;
        // // // let y = f64::acos(f64::cos(rho) / f64::tan(delta) - x);

        // // let prime_y = prime_x * (1.0 - f64::sqrt((1.0 - f64::cos(x)) / (1.0 - f64::cos(x + y))));
        // // let x = ;
        [0.0, 0.0]
    }
}
