use std::f64::consts::PI;
use std::usize;

use crate::models::common::{Position2D, PositionGeo};
use crate::models::quaternion::Quaternion;
use crate::models::vector_3d::Vector3D;
use crate::utils::math::{cos, sin, to_deg, to_rad};
use crate::{traits::polyhedron::Polyhedron, utils::math::pow};

use crate::traits::polyhedron::{ArcLengths, UnitVectors, Vertexes};

/// (1 + sqrt(5)) / 2
pub const GOLDEN_RATIO: f64 = 1.618;

pub const FACES: u8 = 20;

pub const ORIENTATION_LAT: f64 = 31.7174744114611;
pub const ORIENTATION_LON: f64 = 11.20;

#[derive(Default, Debug)]
pub struct Rhombic5x6 {}

impl Polyhedron for Rhombic5x6 {
    fn get_faces(&self) -> u8 {
        FACES
    }
    fn get_planar_vertexes(&self) -> Vec<[(u8, u8); 3]> {
        TRIANGLES.to_vec()
    }
    fn get_indices(&self) -> Vec<[u8; 3]> {
        INDICES.to_vec()
    }
    fn get_triangle_unit_vectors(&self) -> UnitVectors {
        let aux = 1.0 / f64::sqrt(1.0 + pow(self::GOLDEN_RATIO, 2));
        let aux1 = self::GOLDEN_RATIO / f64::sqrt(1.0 + pow(self::GOLDEN_RATIO, 2));

        let a = [0.0, aux, aux1];
        let b = [aux, aux1, 0.0];
        let c = [aux1, 0.0, aux];
        UnitVectors { a, b, c }
    }
    fn get_unit_vectors(&self) -> Vec<Vector3D> {
        // Vertices authalic latitude - 26.565º
        let t = f64::atan(0.5);
        let ty = -t.sin();
        let by = -(-t).sin();
        let tc = t.cos();
        let bc = (-t).cos();

        // normalized radius
        let r = 1.0;

        // area of the icosahedron triangular face
        let s = 2.0 * PI / 5.0;

        let mut vertices = vec![
            Vector3D { x: 0.0, y: 0.0, z: 0.0 };
            20 // Preallocate enough space
        ];

        // North pole
        vertices[0] = Vector3D {
            x: 0.0,
            y: -r,
            z: 0.0,
        };
        // South pole
        vertices[11] = Vector3D {
            x: 0.0,
            y: r,
            z: 0.0,
        };

        let q = Quaternion::yaw_pitch(-ORIENTATION_LON.to_radians(), -ORIENTATION_LAT.to_radians());

        for i in 0..5 {
            let deg: f64 = (-180.0 - 36.0 / 2.0 - 72.0);
            let ta = deg.to_radians() + s * i as f64;
            let ba = ta + s / 2.0;

            // North hemisphere
            vertices[1 + i] = Vector3D {
                x: ta.cos() * r * tc,
                y: ty * r,
                z: ta.sin() * r * tc,
            };

            // South hemisphere
            vertices[6 + i] = Vector3D {
                x: ba.cos() * r * bc,
                y: by * r,
                z: ba.sin() * r * bc,
            };
        }

        for i in 0..12 {
            vertices[i] = q.rotate_vector(vertices[i]);
        }

        vertices.to_vec()
    }

    // to 90 degrees right triangle
    fn get_triangle_arc_lengths(
        &self,
        vector: [f64; 3],
        face_vectors: Vec<Vector3D>,
        face_vertices: [(u8, u8); 3],
    ) -> ArcLengths {
        // let uvs = self.get_triangle_unit_vectors();
        // let dot_ab = 0.0;
        // let ab = f64::acos(dot_ab);
        // let bc = f64::acos(uvs.b[0] * uvs.c[0] + uvs.b[1] * uvs.c[1] + uvs.b[2] * uvs.c[2]);
        // let ac = f64::acos(uvs.a[0] * uvs.c[0] + uvs.a[1] * uvs.c[1] + uvs.a[2] * uvs.c[2]);

        // let lat = to_rad(0.0);
        // let lon = to_rad(0.0);
        // // calculate 3d unit vectors for point P
        // let uv_px = cos(lat) * cos(lon);
        // let uv_py = cos(lat) * sin(lon);
        // let uv_pz = sin(lat);
        // let ap = f64::acos(uvs.a[0] * uv_px + uvs.a[1] * uv_py + uvs.a[2] * uv_pz); //f64::acos(uvs.a[0] * uvp[0] + uvs.a[1] * uvp[1] + uvs.a[2] * uvp[2]);
        // let bp = f64::acos(uvs.b[0] * uv_px + uvs.b[1] * uv_py + uvs.b[2] * uv_pz);
        // let cp = f64::acos(uvs.c[0] * uv_px + uvs.c[1] * uv_py + uvs.c[2] * uv_pz);
        let point_center = self.get_face_center_2d(face_vertices);
        let vector_center =
            self.get_face_center_3d(face_vectors[0], face_vectors[1], face_vectors[2]);

        let p_mid = Position2D::mid(face_vertices[1], face_vertices[2]);
        let v_mid = Vector3D::mid(face_vectors[1],face_vectors[2]);
        ArcLengths {
            ab,
            bc,
            ac,
            ap,
            bp,
            cp,
        }
    }
    fn get_face_center_2d(&self, p: [(u8, u8); 3]) -> Position2D {
        Position2D {
            x: (p[0].0 + p[1].0 + p[2].0) / 3,
            y: (p[0].1 + p[1].1 + p[2].1) / 3,
        }
    }
    fn get_face_center_3d(
        &self,
        vector1: [f64; 3],
        vector2: [f64; 3],
        vector3: [f64; 3],
    ) -> Vector3D {
        Vector3D {
            x: (vector1[0] + vector2[0] + vector3[0]) / 3.0,
            y: (vector1[1] + vector2[1] + vector3[1]) / 3.0,
            z: (vector1[2] + vector2[2] + vector3[2]) / 3.0,
        }
    }
}

// DGGAL configuration
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
];

const INDICES: [[u8; 3]; 20] = [
    // Top triangles
    [0, 1, 2],
    [0, 2, 3],
    [0, 3, 4],
    [0, 4, 5],
    [0, 5, 1],
    // Mirror of Top triangles
    [6, 2, 1],
    [7, 3, 2],
    [8, 4, 3],
    [9, 5, 4],
    [10, 1, 5],
    // Mirror of Bottom triangles
    [2, 6, 7],
    [3, 7, 8],
    [4, 8, 9],
    [5, 9, 10],
    [1, 10, 6],
    // Bottom triangles
    [11, 7, 6],
    [11, 8, 7],
    [11, 9, 8],
    [11, 10, 9],
    [11, 6, 10],
];
