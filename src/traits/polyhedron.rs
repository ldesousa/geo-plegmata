use crate::models::{common::PositionGeo, vector_3d::Vector3D};

pub trait Polyhedron {
    // fn new(self) ;
    // fn golden_ratio(&self) -> f64;
    // fn spherical_distance(&self) -> f64;
    // fn plane_angle(&self) -> f64;
    fn get_faces(&self) -> u8;
    fn get_planar_vertexes(&self) -> Vec<[(u8, u8); 3]>;
    fn get_unit_vectors(&self) -> Vec<Vector3D>;
    fn get_triangle_unit_vectors(&self) -> UnitVectors;
    fn get_triangle_arc_lengths(&self, vector: [f64; 3]) -> ArcLengths;
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
