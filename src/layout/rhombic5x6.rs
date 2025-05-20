// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::models::common::Position2D;
use crate::traits::layout::Layout;

#[derive(Default, Debug)]
pub struct Rhombic5x6 {}

impl Layout for Rhombic5x6 {
    fn face_center(&self, p: [(u8, u8); 3]) -> Position2D {
        Position2D {
            x: f64::from((p[0].0 + p[1].0 + p[2].0) / 3),
            y: f64::from((p[0].1 + p[1].1 + p[2].1) / 3),
        }
    }

    fn grid_size(&self) -> (usize, usize) {
        (5, 6)
    }
    fn vertices(&self) -> Vec<[(u8, u8); 3]> {
        TRIANGLES.to_vec()
    }
}

// DGGAL configuration for 5x6
const TRIANGLES: [[(u8, u8); 3]; 20] = [
    // Top triangles
    [(1, 0), (0, 0), (1, 1)],
    [(2, 1), (1, 1), (2, 2)],
    [(3, 2), (2, 2), (3, 3)],
    [(4, 3), (3, 3), (4, 4)],
    [(5, 4), (4, 4), (5, 5)],
    // Mirror of Top triangles
    [(0, 1), (1, 1), (0, 0)],
    [(1, 2), (2, 2), (1, 1)],
    [(2, 3), (3, 3), (2, 2)],
    [(3, 4), (4, 4), (3, 3)],
    [(4, 5), (5, 5), (4, 4)],
    // Mirror of Bottom triangles
    [(1, 1), (0, 1), (1, 2)],
    [(2, 2), (1, 2), (2, 3)],
    [(3, 3), (2, 3), (3, 4)],
    [(4, 4), (3, 4), (4, 5)],
    [(5, 5), (4, 5), (5, 6)],
    // Bottom triangles
    [(0, 2), (1, 2), (0, 1)],
    [(1, 3), (2, 3), (1, 2)],
    [(2, 4), (3, 4), (2, 3)],
    [(3, 5), (4, 5), (3, 4)],
    [(4, 6), (5, 6), (4, 5)],
];
