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
use geoplegma::types::Point;
use geoplegma::types::RefinementLevel;
use gp_proj::projections::projections::{traits::Projection, vgc::Vgc};

pub struct IVEA3HBary {
    refinement_level: RefinementLevel,
    denominator: u32,
}

#[allow(dead_code)]
impl IVEA3HBary {
    pub fn new(refinement_level: RefinementLevel) -> Self {
        Self {
            refinement_level: refinement_level,
            denominator: Self::compute_denom(refinement_level),
        }
    }

    pub fn set_refinement_level(&mut self, refinement_level: RefinementLevel) {
        self.refinement_level = refinement_level;
        self.denominator = Self::compute_denom(refinement_level);
    }

    // Denominator is a power of the APERTURE, but only increases every other resolution.
    fn compute_denom(refinement_level: RefinementLevel) -> u32 {
        Self::APERTURE.pow((refinement_level.get() as u32 + refinement_level.get() as u32 % 2) / 2)
    }

    // Bundles barycentric coordinates on a icosahedron face into a 64-bit index
    fn bundle_zone_id(&self, i: u32, j: u32, face: i32) -> u64 {
        i as u64 +                         // i
        j as u64 * 2_u64.pow(26) +    // j
        face as u64 * 2_u64.pow(52) + // face
        self.refinement_level.get() as u64 * 2_u64.pow(57)
    }

    // Unbundles a 64-bit zone identifier into barycentric coordinates and a face index
    pub fn unbundle_zone_id(zone_id: u64) -> (u32, u32, i32, RefinementLevel) {
        let bary_i: u32 = (zone_id % 2_u64.pow(26)) as u32;
        let mut tail: u64 = zone_id / 2_u64.pow(26);
        let bary_j: u32 = (tail % 2_u64.pow(26)) as u32;
        tail = tail / 2_u64.pow(26);
        let face: i32 = (tail % 2_u64.pow(5)) as i32;
        let level = RefinementLevel::new((tail / 2_u64.pow(5)) as i32).expect("REASON");

        (bary_i, bary_j, face, level)
    }

    // Computes distance with barycentric coordinates defined by an equilateral triangle.
    fn bary_distance(i1: f64, j1: f64, i2: f64, j2: f64) -> f64 {
        let d1 = i1 - j1;
        let d2 = i2 - j2;
        d1.powi(2) + d2.powi(2) + d1 * d2
    }

    fn find_nearest_zone_centre(&self, bary: (f64, f64)) -> (u32, u32) {
        let mut zone_centre = (1, 1); // the result

        let mut candidates: Vec<(u32, u32)> = Vec::new();

        let j_down = (bary.1 * self.denominator as f64).floor() as u32;
        let j_up = (bary.1 * self.denominator as f64).ceil() as u32;

        // Odd case
        if (self.refinement_level.get() % 2) > 0 {
            let start_down = j_down % Self::APERTURE;
            let start_up = j_up % Self::APERTURE;
            let num_hops =
                (bary.0 * self.denominator as f64 / Self::APERTURE as f64).floor() as u32; // integer division
            let i_down: u32 = start_down + num_hops * Self::APERTURE;
            let i_up: u32 = start_up + num_hops * Self::APERTURE;
            candidates.push((i_down, j_down));
            candidates.push((i_down + Self::APERTURE, j_down));
            candidates.push((i_up, j_up));
            candidates.push((i_up + Self::APERTURE, j_up));
            println!("Candidates i {} {}", i_down, i_up);
        }
        // Even case
        else {
            let i_down: u32 = (bary.0 * self.denominator as f64).floor() as u32;
            let i_up: u32 = (bary.0 * self.denominator as f64).ceil() as u32;
            candidates.push((i_down, j_down));
            candidates.push((i_down, j_up));
            candidates.push((i_up, j_down));
            candidates.push((i_up, j_up));
            println!("Candidates i {} {}", i_down, i_up);
        }

        println!("Candidates {:?}", candidates);

        // Find closest cell centre
        let mut current_dist = f64::MAX;
        while candidates.len() > 0 {
            let centre = candidates.pop().unwrap();
            let dist = Self::bary_distance(
                f64::from(centre.0) / f64::from(self.denominator),
                bary.0,
                f64::from(centre.1) / f64::from(self.denominator),
                bary.1,
            );
            if dist < current_dist {
                current_dist = dist;
                zone_centre = centre;
            }
        }

        println!("Winner {:?}", zone_centre);
        return zone_centre;
    }

    // Determines face to be enconded in the index for edge cases, i.e. cells/zones spaning two or more
    // icosahedron faces. Guarantees each cell/zone has only one index.
    fn edge_cases(&self, mut i: u32, mut j: u32, mut face: i32) -> (u32, u32, i32) {
        let mut zero = false;
        let mut swap = false;
        let k = self.denominator - i - j;

        // top-most row of faces
        if face < 10 && (face % 2) > 0 {
            if j == self.denominator {
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
            if j == self.denominator
            // bottom pentagon
            {
                face = face + 10;
                zero = true;
            } else if k == self.denominator
            // left-most pentagon
            {
                face = face - 1;
                zero = true;
            } else if i == self.denominator
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
            if j == self.denominator
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
            } else if i == self.denominator
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
            if j == self.denominator {
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
        _refinement_level: RefinementLevel,
        point: Point,
        //config: Option<DggrsApiConfig>,
    ) -> u64 {
        // Icosahedron selected by default
        let bary = Vgc::default().geo_to_barycentric(vec![point], None, None, None);

        println!("Barycentric: {:?}", bary);

        let zone_centre =
            IVEA3HBary::find_nearest_zone_centre(self, (bary[0].coords.x, bary[0].coords.y));

        let unique =
            IVEA3HBary::edge_cases(self, zone_centre.0, zone_centre.1, bary[0].face as i32);
        println!("After edge cases: {:?}", unique);

        // Bundle index into 64 bit
        return IVEA3HBary::bundle_zone_id(self, unique.0, unique.1, unique.2);
    }
}

#[cfg(test)]
#[path = "ivea3h_bary_test.rs"]
mod ivea3h_bary_test;
