// Copyright 2025 contributors to the GeoPlegmata project.
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

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
