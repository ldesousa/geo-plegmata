// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Luís Moreira de Sousa, Técnico, ULisboa
// (luis.moreira.de.sousa [at] tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::sys_api::DggrsSysApi;
use geo::Point;
//use api::error::DggrsError;
use geoplegma::types::RefinementLevel; //, Zones};
use gp_proj::{
    projections::{
        polyhedron::{icosahedron::new, spherical_geometry::barycentric_coordinates},
        projections::{traits::Projection, vgc::Vgc},
    },
    utils::shape::cartesian_to_barycentric,
};
pub struct IVEA3HBary {}

#[allow(dead_code)]
impl IVEA3HBary {
    // Denominator is a power of the APERTURE, but only increases every other resolution.
    fn compute_denom(refinement_level: RefinementLevel) -> u32 {
        return Self::APERTURE
            .pow((refinement_level.get() as u32 + refinement_level.get() as u32 % 2) / 2)
            as u32;
    }

    // Fake method for the time being - then use the Projection module
    pub fn project(point: Point) -> (f64, f64, f64) {
        return (point.x(), point.y(), (1.0 - point.x() - point.y()));
    }

    fn bundle_index(_i: i32, _j: i32, _refinement_level: RefinementLevel, _face: i32) {
        // do something
    }

    // Computes distance with barycentric coordinates defined by an equilateral triangle.
    fn bary_distance(i1: f64, j1: f64, i2: f64, j2: f64) -> f64 {
        let d1 = i1 - j1;
        let d2 = i2 - j2;
        return d1.powi(2) + d2.powi(2) + d1 * d2;
    }

    // Determines face to be enconded in index for edge cases, i.e. cells/zones spaning two or more
    // icosahedron faces. Guarantees each cell/zone has only one index.
    fn edge_cases(mut i: u32, mut j: u32, mut face: i32, denom: u32) -> (u32, u32, i32) {
        let mut zero = false;
        let mut swap = false;
        let k = denom - i - j;

        // top-most row of faces
        if face < 10 && (face % 2) > 0 {
            if j == denom {
                face = 1;
            }
            // top-most pentagon
            else if k == 0
            // shared cell on the right-edge: moves to right
            {
                face = face + 2;
                if face > 9 {
                    face = 1;
                } // wrap around
                i = 0;
            }
        }
        // middle row of faces pointing "downwards"
        else if face < 11 && (face % 2) == 0 {
            if j == denom
            // bottom pentagon
            {
                face = face + 10;
                zero = true;
            } else if k == denom
            // left-most pentagon
            {
                face = face - 1;
                zero = true;
            } else if i == denom
            // right-most pentagon
            {
                face = face + 1;
                if face > 10 {
                    face = 1;
                }
                zero = true;
            } else if j == 0 {
                face = face - 1;
            }
            // top edge
            else if k == 0
            // right edge
            {
                face = face + 9;
                swap = true;
            }
        }
        // middle row of faces pointing "upwards"
        else if face < 20 && (face % 2) > 0 {
            if j == denom
            // top-most pentagon
            {
                face = face - 8;
                if face > 9 {
                    face = 1;
                } // wrap around
                zero = true;
            } else if k == 1
            // left-most pentagon
            {
                face = face + 1;
                zero = true;
            } else if i == denom
            // right-most pentagon
            {
                face = face + 3;
                if face > 20 {
                    face = 12;
                } // wrap around
                zero = true;
            } else if j == 0 {
                face = face + 1;
            }
            // bottom edge
            else if k == 0
            // right edge
            {
                face = face - 7;
                swap = true;
            }
        } else
        // botom row of faces
        {
            if j == denom {
                face = 12;
            }
            // bottom pentagon
            else if k == 0
            // right edge
            {
                face = face + 2;
                if face > 20 {
                    face = 12;
                } // wrap around
                i = 0;
            }
        }

        if zero {
            i = 0;
            j = 0;
        }
        if swap {
            let temp = j;
            j = i;
            i = temp;
        }
        return (i, j, face);
    }
}

impl DggrsSysApi for IVEA3HBary {
    const APERTURE: u32 = 3;

    fn zone_from_point(
        &self,
        refinement_level: RefinementLevel,
        point: Point,
        //config: Option<DggrsApiConfig>,
    ) -> u64 {
        //        let bary = IVEA3HBary::project(point);
        let projection = Vgc;
        let icosahedron = new();
        let projected = projection.geo_to_cartesian(vec![point], Some(&icosahedron), None);
        let triangle = projected[0].triangle;
        let bary_coords = cartesian_to_barycentric(
            (
                projected[0].coords.x / 6371007.181,
                projected[0].coords.y / 6371007.181,
            ),
            triangle[0],
            triangle[1],
            triangle[2],
        );
        let bary = (bary_coords.0, bary_coords.2);

        let denom = IVEA3HBary::compute_denom(refinement_level);

        let mut zone_centre = (1 as u32, 1 as u32); // the result

        let mut candidates: Vec<(u32, u32)> = Vec::new();

        let j_down = (bary.1 * denom as f64).floor() as u32;
        let j_up = (bary.1 * denom as f64).ceil() as u32;

        // Odd case
        if (refinement_level.get() % 2) > 0 {
            let start_down = j_down % Self::APERTURE;
            let start_up = j_up % Self::APERTURE;
            let num_hops = (bary.0 * denom as f64 / Self::APERTURE as f64).floor() as u32; // integer division
            let i_down: u32 = start_down + num_hops * Self::APERTURE;
            let i_up: u32 = start_up + num_hops * Self::APERTURE;
            candidates.push((i_down, j_down));
            candidates.push((i_down + Self::APERTURE, j_down));
            candidates.push((i_up, j_up));
            candidates.push((i_up + Self::APERTURE, j_up));
        }
        // Even case
        else {
            let i_down: u32 = (bary.0 * denom as f64).floor() as u32;
            let i_up: u32 = (bary.0 * denom as f64).ceil() as u32;
            candidates.push((i_down, j_down));
            candidates.push((i_down, j_up));
            candidates.push((i_up, j_down));
            candidates.push((i_up, j_up));
        }

        // Find closest cell centre
        let mut current_dist = f64::MAX;
        while candidates.len() > 0 {
            let centre = candidates.pop().unwrap();
            let dist = Self::bary_distance(
                f64::from(centre.0) / f64::from(denom),
                bary.0,
                f64::from(centre.1) / f64::from(denom),
                bary.1,
            );
            if dist < current_dist {
                current_dist = dist;
                zone_centre = centre;
            }
        }

        // Bundle coords into index
        // bundle_index(zone_centre.0, zone_centre.1, refinement_level, bary.3);
        // return zone_centre;
        let face: i32 = 10;
        let unique = IVEA3HBary::edge_cases(zone_centre.0, zone_centre.1, face, denom);
        return unique.0 as u64 +                        // i
               unique.1 as u64 * 2_u64.pow(26) as u64 + // j
               unique.2 as u64 * 2_u64.pow(52) as u64 + // face
               refinement_level.get() as u64 * 2_u64.pow(57) as u64;
    }
}

#[cfg(test)]
mod tests {

    use crate::impls::ivea3h_bary::IVEA3HBary;
    use crate::sys_api::DggrsSysApi;
    use api::models::common::RefinementLevel;
    use geo::Point;

    #[test]
    fn test_zone_from_point() {
        let system = IVEA3HBary {};

        //let mut level = RefinementLevel::new(3).unwrap();
        //let mut zone = system.zone_from_point(level, Point::new(0.45,0.22));
        //assert_eq!(zone.0, 4);
        //assert_eq!(zone.1, 1);

        level = RefinementLevel::new(4).unwrap();
        zone = system.zone_from_point(level, Point::new(0.21, 0.64));
        let bary_i = zone % 2_u64.pow(26);
        let mut tail: u64 = zone / 2_u64.pow(26);
        let bary_j = tail % 2_u64.pow(26);
        tail = tail / 2_u64.pow(26);
        let face = tail % 2_u64.pow(5);
        let level = tail / 2_u64.pow(5);
        assert_eq!(bary_i, 2);
        assert_eq!(bary_j, 6);
        assert_eq!(face, 10);
        assert_eq!(level, 4);
    }
}
