use crate::models::common::Position;

use super::polyhedron::Polyhedron;

pub trait Projection {
    fn forward(&self, positions: Vec<Position>, shape: &dyn Polyhedron) -> [f64; 2];
    fn inverse(&self) -> String;
}
