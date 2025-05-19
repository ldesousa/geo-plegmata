use crate::models::{common::Position2D, vector_3d::Vector3D};

use super::layout::Layout;

pub trait Polyhedron {
    fn faces(&self) -> u8;
    fn indices(&self) -> Vec<[u8; 3]>;
    fn unit_vectors(&self) -> Vec<Vector3D>;
    fn triangles(
        &self,
        layout: &dyn Layout,
        vector: Vector3D,
        face_vectors: Vec<Vector3D>,
        face_vertices: [(u8, u8); 3],
    ) -> ([Vector3D; 3], [Position2D; 3]);
    fn triangle_arc_lengths(&self, triangle: [Vector3D; 3],
        vector: Vector3D) -> ArcLengths;
    fn face_center(
        &self,
        vector1: Vector3D,
        vector2: Vector3D,
        vector3: Vector3D,
    ) -> Vector3D;
    fn is_point_in_triangle(&self, point: Vector3D, triangle: Vec<Vector3D>) -> bool;
    fn angle_between_unit(&self, u: Vector3D, v: Vector3D) -> f64;
}

#[derive(Default, Debug)]
pub struct ArcLengths {
    pub ab: f64,
    pub bc: f64,
    pub ac: f64,
    pub ap: f64,
    pub bp: f64,
    pub cp: f64,
}
