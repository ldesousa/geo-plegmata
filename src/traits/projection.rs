use crate::models::common::{Position2D, PositionGeo};

use super::polyhedron::Polyhedron;

pub trait Projection {
    fn forward(&self, positions: Vec<PositionGeo>, shape: &dyn Polyhedron) -> Vec<Position2D>;
    fn inverse(&self) -> String;
}
