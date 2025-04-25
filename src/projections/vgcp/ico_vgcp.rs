use crate::{
    models::common::Position,
    shape::icosahedron::{consts::SPHERICAL_DISTANCE, ArcLengths, Icosahedron},
    traits::{polyhedron::Polyhedron, projection::Projection},
};

use super::vgcp::Vgcp;

// Implementation for Icosahedron Vertex Great Circle Projection.
pub struct Ivgcp {}

impl Projection for Ivgcp {
    fn forward(positions: Vec<Position>, shape: &dyn Polyhedron) -> [f64; 2] {
        // let ico = Icosahedron::new();
        // let ArcLengths {
        //     ab,
        //     bp,
        //     ap,
        //     bc,
        //     ac,
        //     cp,
        // } = ico.triangle_arcs;
        // let arc_p = SPHERICAL_DISTANCE;
        // let v2d = ico.triangle_vertexes;

        // self::new(<sq)
        [0.0, 0.0]
    }
    fn inverse(&self) -> String {
        todo!()
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