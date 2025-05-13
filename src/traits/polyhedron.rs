use crate::models::{common::PositionGeo, vector_3d::Vector3D};

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
    fn get_triangle_arc_lengths(&self, vector: [f64; 3], 
        shape_vectors: Vec<Vector3D>,

        face_vertices: [(u8, u8); 3]
    ) -> ArcLengths;
    fn get_face_center_2d(&self,face_vertices: [(u8, u8); 3]) -> PositionGeo;
    fn get_face_center_3d(&self,  vector1: [f64; 3], vector2: [f64; 3], vector3: [f64; 3]) -> Vector3D;
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
