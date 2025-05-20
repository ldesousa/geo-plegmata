// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::models::common::Position2D;
pub trait Layout {
    fn face_center(&self, vertices: [(u8, u8); 3]) -> Position2D;
    fn grid_size(&self) -> (usize, usize);
    fn vertices(&self) -> Vec<[(u8, u8); 3]>;
}
