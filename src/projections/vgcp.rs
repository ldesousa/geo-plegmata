use std::f64::consts::{E, PI};

use crate::{
    models::common::Position,
    traits::{polyhedron::{ArcLengths, Polyhedron}, projection::Projection}, utils::math::{cos, pow, sin, tan, to_rad},
};

// use super::vgcp::Vgcp;

/// Implementation for Icosahedron Vertex Great Circle Projection.
/// vgcp - Vertex Great Circle Projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub struct Vgcp;

- conseguir os vertices dos triangulos, achar a configuração do Icosahedron
- Achar onde os pontos se encontram no triangulo
https://chatgpt.com/c/68091cde-4428-8002-beb3-6713a35494ec

impl Projection for Vgcp {
    fn forward(&self, positions: Vec<Position>, shape: &dyn Polyhedron) -> [f64; 2] {
        let ArcLengths {
            ab,
            bp,
            ap,
            bc,
            ac,
            cp,
        } = shape.triangle_arc_lengths();
        // let arc_p = SPHERICAL_DISTANCE;
        let v2d = shape.planar_vertexes();
        let uvs = shape.unit_vectors();

        // angle ρ
        let rho: f64 = f64::acos(cos(ap) - cos(ab) * cos(ap)) / (sin(ab) * sin(ap));

        // /// @TODO: ADD TO CONSTANTS OF ICOSAHEDRON
        let angle_beta: f64 = to_rad(36.0);
        let angle_phi: f64 = to_rad(60.0);
        let angle_bac: f64 = PI/2.0;

        // /// 1. Calculate delta (δ)
        let delta = f64::acos(f64::sin(rho) * f64::cos(ab));

        // /// 2. Calculate u
        let uv = (angle_beta + angle_phi - rho - delta) / (angle_beta + angle_phi - PI / 2.0);

        let cosXpY;
        if rho <= pow(E, -9) {
            cosXpY = cos(ab);
        } else {
            cosXpY = 1.0 / (tan(rho) * tan(delta))
        }

        let xy = f64::sqrt((1.0 - cos(ap)) / (1.0 - cosXpY));

        // entre o A e o C que dá o ponto D
        let pdi_x = v2d.c[0] + (v2d.a[0] - v2d.c[0]) * uv;
        let pdi_y = v2d.c[1] + (v2d.a[1] - v2d.c[1]) * uv;

        // entre o D e o B que dá o ponto P
        let pdi_x = pdi_x + (pdi_x - v2d.b[0]) * xy;
        let pdi_y = pdi_y + (pdi_y - v2d.b[1]) * xy;

        [pdi_x, pdi_y]
    }
    fn inverse(&self) -> String {
        "todo!()".to_string()
    }
}

// impl Projection<Tetra> for Ivgcp {
//     fn forward(positions: Vec<Position>, shape: &Tetra) -> [f64; 2] {
//         todo!()
//     }

//     fn inverse(&self) -> String {
//         todo!()
//     }
// }
