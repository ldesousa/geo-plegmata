use std::usize;

use crate::models::common::PositionGeo;
use crate::utils::math::{cos, sin, to_deg, to_rad};
use crate::{traits::polyhedron::Polyhedron, utils::math::pow};

use crate::traits::polyhedron::{ArcLengths, UnitVectors, Vertexes};

/// (1 + sqrt(5)) / 2
pub const GOLDEN_RATIO: f64 = 1.618;

pub const FACES: u8 = 20;

#[derive(Default, Debug)]
pub struct Rhombic5x6 {}

impl Polyhedron for Rhombic5x6 {
    fn get_faces(&self) -> u8 {
        FACES
    }
    fn get_planar_vertexes(&self) -> Vec<[(u8, u8); 3]> {
        TRIANGLES.to_vec()
    }
    fn get_triangle_unit_vectors(&self) -> UnitVectors {
        let aux = 1.0 / f64::sqrt(1.0 + pow(self::GOLDEN_RATIO, 2));
        let aux1 = self::GOLDEN_RATIO / f64::sqrt(1.0 + pow(self::GOLDEN_RATIO, 2));

        let a = [0.0, aux, aux1];
        let b = [aux, aux1, 0.0];
        let c = [aux1, 0.0, aux];

        UnitVectors { a, b, c }
    }

    // to 90 degrees right triangle
    fn get_triangle_arc_lengths(&self, p: &PositionGeo) -> ArcLengths {
        let uvs = self.get_triangle_unit_vectors();
        let dot_ab = 0.0;
        let ab = f64::acos(dot_ab);
        let bc = f64::acos(uvs.b[0] * uvs.c[0] + uvs.b[1] * uvs.c[1] + uvs.b[2] * uvs.c[2]);
        let ac = f64::acos(uvs.a[0] * uvs.c[0] + uvs.a[1] * uvs.c[1] + uvs.a[2] * uvs.c[2]);

        let lat = to_rad(p.lat);
        let lon = to_rad(p.lon);
        // calculate unit vectors for point P
        let uv_px = cos(lat) * cos(lon);
        let uv_py = cos(lat) * sin(lon);
        let uv_pz = sin(lat);

        let ap = f64::acos(uvs.a[0] * uv_px + uvs.a[1] * uv_py + uvs.a[2] * uv_pz); //f64::acos(uvs.a[0] * uvp[0] + uvs.a[1] * uvp[1] + uvs.a[2] * uvp[2]);
        let bp = f64::acos(uvs.b[0] * uv_px + uvs.b[1] * uv_py + uvs.b[2] * uv_pz);
        let cp = f64::acos(uvs.c[0] * uv_px + uvs.c[1] * uv_py + uvs.c[2] * uv_pz);

        ArcLengths {
            ab,
            bc,
            ac,
            ap,
            bp,
            cp,
        }
    }
}

const TRIANGLES: [[(u8, u8); 3]; 20] = [
    // Top triangles
    [(1, 0), (0, 0), (1, 1)],
    [(2, 1), (1, 1), (2, 2)],
    [(3, 2), (2, 2), (3, 3)],
    [(4, 3), (3, 3), (4, 4)],
    [(5, 4), (4, 4), (5, 5)],
    // Mirror of Top triangles
    [(0, 1), (1, 1), (0, 0)],
    [(1, 2), (2, 2), (1, 1)],
    [(2, 3), (3, 3), (2, 2)],
    [(3, 4), (4, 4), (3, 3)],
    [(4, 5), (5, 5), (4, 4)],
    // Mirror of Bottom triangles
    [(1, 1), (0, 1), (1, 2)],
    [(2, 2), (1, 2), (2, 3)],
    [(3, 3), (2, 3), (3, 4)],
    [(4, 4), (3, 4), (4, 5)],
    [(5, 5), (4, 5), (5, 6)],
    // Bottom triangles
    [(0, 2), (1, 2), (0, 1)],
    [(1, 3), (2, 3), (1, 2)],
    [(2, 4), (3, 4), (2, 3)],
    [(3, 5), (4, 5), (3, 4)],
    [(4, 6), (5, 6), (4, 5)],
]
;
