use crate::models::{common::{Position2D, PositionGeo}, vector_3d::Vector3D};

pub trait Polyhedron {
    // fn new(self) ;
    // fn golden_ratio(&self) -> f64;
    // fn spherical_distance(&self) -> f64;
    // fn plane_angle(&self) -> f64;
    fn get_faces(&self) -> u8;
    fn get_planar_vertexes(&self) -> Vec<[(u8, u8); 3]>;
    fn get_indices(&self) -> Vec<[u8; 3]>;
    fn get_unit_vectors(&self) -> Vec<Vector3D>;
    fn get_triangle_unit_vectors(&self) -> UnitVectors;
    fn get_triangle_arc_lengths(
        &self,
        vector: Vector3D,
        shape_vectors: Vec<Vector3D>,

        face_vertices: [(u8, u8); 3],
    ) -> ArcLengths;
    fn get_face_center_2d(&self, face_vertices: [(u8, u8); 3]) -> Position2D;
    fn get_face_center_3d(
        &self,
        vector1: Vector3D,
        vector2: Vector3D,
        vector3: Vector3D,
    ) -> Vector3D;
    fn is_point_in_triangle(&self, point: Vector3D, triangle_3d: Vec<Vector3D>) -> bool;
    fn angle_between_unit(&self, u: Vector3D, v: Vector3D) -> f64;
}

#[derive(Default, Debug)]
pub struct UnitVectors {
    pub a: [f64; 3],
    pub b: [f64; 3],
    pub c: [f64; 3],
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
#[derive(Default, Debug)]
pub struct Vertexes {
    pub a: [f64; 2],
    pub b: [f64; 2],
    pub c: [f64; 2],
}
