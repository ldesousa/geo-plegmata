use std::f64::consts::PI;
use crate::models::common::{Position2D};
use crate::models::plane::Plane;
use crate::models::quaternion::Quaternion;
use crate::models::vector_3d::Vector3D;
use crate::{traits::polyhedron::Polyhedron, utils::math::pow};

use crate::traits::polyhedron::{ArcLengths, UnitVectors};

/// (1 + sqrt(5)) / 2
pub const GOLDEN_RATIO: f64 = 1.618;

pub const FACES: u8 = 20;

pub const ORIENTATION_LAT: f64 = 31.7174744114611;
pub const ORIENTATION_LON: f64 = 11.20;

#[derive(Default, Debug)]
pub struct Icosahedron {}

impl Polyhedron for Icosahedron {
    fn get_faces(&self) -> u8 {
        FACES
    }
    // for layout
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
            12 // Preallocate enough space
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
    /// 1. Compute center 3D vector of face
    /// 2. Compute center 2D point of face
    /// 3. Check which sub-triangle (out of 3) v falls into:
    ///     a. v2-v3
    ///     b. v3-v1
    ///     c. v1-v2
    /// 4. For that sub-triangle, compute midpoint (vMid, pMid)
    /// 5. Test which sub-sub-triangle v is in (with vCenter + vMid + corner)
    /// 6. Set the triangle vertex indices: [va, vb, vc] = [0, 1, 2]
    /// 7. Normalize vCenter, vMid
    /// 8. Call forwardPointInSDTTriangle(v, ... -> out)
    fn get_triangle_arc_lengths(
        &self,
        vector: Vector3D,
        face_vectors: Vec<Vector3D>,
        face_vertices: [(u8, u8); 3],
    ) -> ArcLengths {
        let [p1, p2, p3] = face_vertices;
        let p1 = Position2D::from_tuple(p1);
        let p2 = Position2D::from_tuple(p2);
        let p3 = Position2D::from_tuple(p3);
        let (v1, v2, v3) = (face_vectors[0], face_vectors[1], face_vectors[2]);
        let point_center = self.get_face_center_2d(face_vertices);
        let mut vector_center = self.get_face_center_3d(v1, v2, v3);

        // let mut p_mid = Position2D::mid(face_vertices[1], face_vertices[2]);
        // let mut v_mid = Vector3D::mid(face_vectors[1], face_vectors[2]);
        let (mut p_mid, mut v_mid, corner): (Position2D, Vector3D, (Vector3D, Position2D)) =
            if self.is_point_in_triangle(vector, vec![vector_center, v2, v3]) {
                let p_mid = Position2D::mid(p2.clone(), p3.clone());
                let v_mid = Vector3D::mid(v2, v3);
                if self.is_point_in_triangle(vector, vec![vector_center, v_mid, v3]) {
                    (p_mid, v_mid, (v3, p3))
                } else {
                    (p_mid, v_mid, (v2, p2))
                }
            } else if self.is_point_in_triangle(vector, vec![vector_center, v3, v1]) {
                let p_mid = Position2D::mid(p3.clone(), p1.clone());
                let v_mid = Vector3D::mid(v3, v1);
                if self.is_point_in_triangle(vector, vec![vector_center, v_mid, v3]) {
                    (p_mid, v_mid, (v3, p3))
                } else {
                    (p_mid, v_mid, (v1, p1))
                }
            } else {
                let p_mid = Position2D::mid(p1.clone(), p2.clone());
                let v_mid = Vector3D::mid(v1, v2);
                if self.is_point_in_triangle(vector, vec![vector_center, v_mid, v2]) {
                    (p_mid, v_mid, (v2, p2))
                } else {
                    (p_mid, v_mid, (v1, p1))
                }
            };

        vector_center = vector_center.normalize();
        v_mid = v_mid.normalize();

        // Vertex indices are [0, 1, 2]
        // Vertices for the 3D triangle that we want (v_mid: B, corner.0: A, v_center: C)
        // let v3d = [v_mid, corner.0, vector_center];
        // Vertices for the 2D triangle that we want
        // let p2d = [p_mid, corner.1, point_center];

        ArcLengths {
            ab: self.angle_between_unit(corner.0, v_mid),
            bc: self.angle_between_unit(v_mid, vector_center),
            ac: self.angle_between_unit(corner.0, vector_center),
            ap: self.angle_between_unit(corner.0, vector),
            bp: self.angle_between_unit(v_mid, vector),
            cp: self.angle_between_unit(vector_center, vector),
        }
    }

    /// DGGAL based code
    /// - Triangle edges are < 90°
    /// - Builds three planes from the triangle's edges
    /// - Dot test: For each plane, compute the signed distance from v3D to the plane.
    /// - If the sign of this distance differs across planes, the point lies outside.
    fn is_point_in_triangle(&self, point: Vector3D, triangle_3d: Vec<Vector3D>) -> bool {
        if self.angle_between_unit(point, triangle_3d[0]) > PI / 2.0 {
            return false;
        }

        let planes = [
            Plane::from_points(
                Vector3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                triangle_3d[0],
                triangle_3d[1],
            ),
            Plane::from_points(
                Vector3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                triangle_3d[1],
                triangle_3d[2],
            ),
            Plane::from_points(
                Vector3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                triangle_3d[2],
                triangle_3d[0],
            ),
        ];

        let mut sign = 0;

        for plane in &planes {
            let d = plane.signed_distance(point);
            if d.abs() > 1e-9 {
                let s = d.signum() as i32;
                if sign != 0 && s != sign {
                    return false;
                }
                sign = s;
            }
        }

        true
    }
    /// Numerically stable angle between two unit vectors
    /// It uses the identity: θ = 2⋅arcsin(∥v−u∥​ / 2)
    fn angle_between_unit(&self, u: Vector3D, v: Vector3D) -> f64 {
        // angle > 90º
        if u.dot(v) < 0.0 {
            let s = u.neg().subtract(v).length() / 2.0;
            PI - 2.0 * s.clamp(-1.0, 1.0).asin()
        } else {
            let s = v.subtract(u).length() / 2.0;
            2.0 * s.clamp(-1.0, 1.0).asin()
        }
    }
    // for layout
    fn get_face_center_2d(&self, p: [(u8, u8); 3]) -> Position2D {
        Position2D {
            x: f64::from((p[0].0 + p[1].0 + p[2].0) / 3),
            y: f64::from((p[0].1 + p[1].1 + p[2].1) / 3),
        }
    }
    fn get_face_center_3d(
        &self,
        vector1: Vector3D,
        vector2: Vector3D,
        vector3: Vector3D,
    ) -> Vector3D {
        Vector3D {
            x: (vector1.x + vector2.x + vector3.x) / 3.0,
            y: (vector1.y + vector2.y + vector3.y) / 3.0,
            z: (vector1.z + vector2.z + vector3.z) / 3.0,
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
