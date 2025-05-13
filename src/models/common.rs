use crate::models::dggrid::CellID;
use geo::{Point, Polygon};

#[derive(Debug)]
pub struct CellGEO {
    pub id: CellID,
    pub region: Polygon,
    pub center: Point,
    pub vertex_count: u32,
    pub children: Option<Vec<String>>,
    pub neighbors: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct CellsGEO {
    pub cells: Vec<CellGEO>,
}
