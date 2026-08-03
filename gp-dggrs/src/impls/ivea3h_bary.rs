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
use geoplegma::types::RefinementLevel; 
use gp_proj::{
    projections::{
        polyhedron::{Orientation, icosahedron::new},
        projections::{traits::Projection, vgc::Vgc},
    },
    utils::shape::cartesian_to_barycentric,
};

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
        return Self::APERTURE
            .pow((refinement_level.get() as u32 + refinement_level.get() as u32 % 2) / 2)
            as u32;
    }

    // Fake method for the time being - then use the Projection module
    pub fn project(point: Point) -> (f64, f64, f64) {
        return (point.x(), point.y(), (1.0 - point.x() - point.y()));
    }

    // Bundles barycentric coordinates on a icosahedron face into a 64-bit index
    fn bundle_zone_id(&self, i: u32, j: u32, face: i32) -> u64 {
        
        return i as u64 +                           // i
               j as u64 * 2_u64.pow(26) as u64 +    // j
               face as u64 * 2_u64.pow(52) as u64 + // face
               self.refinement_level.get() as u64 * 2_u64.pow(57) as u64;

    }

    // Unbundles a 64-bit zone identifier into barycentric coordinates and a face index
    pub fn unbundle_zone_id(zone_id: u64) -> (u64, u64, u64, u64) {

        let bary_i = zone_id % 2_u64.pow(26);
        let mut tail:u64 = zone_id / 2_u64.pow(26);
        let bary_j = tail % 2_u64.pow(26); 
        tail = tail / 2_u64.pow(26);
        let face = tail % 2_u64.pow(5);
        let level = tail / 2_u64.pow(5);
    
        return (bary_i, bary_j, face, level);
    }

    // Computes distance with barycentric coordinates defined by an equilateral triangle.
    fn bary_distance(i1: f64, j1: f64, i2: f64, j2: f64) -> f64 {
        let d1 = i1 - j1;
        let d2 = i2 - j2;
        return d1.powi(2) + d2.powi(2) + d1 * d2;
    }

    fn find_nearest_zone_centre(&self, bary: (f64, f64)) -> (u32, u32) {

        let mut zone_centre = (1 as u32, 1 as u32); // the result

        let mut candidates: Vec<(u32, u32)> = Vec::new();
        
        let j_down = (bary.1 * self.denominator as f64).floor() as u32;
        let j_up = (bary.1 * self.denominator as f64).ceil() as u32;

        // Odd case
        if (self.refinement_level.get() % 2) > 0 {
            let start_down = j_down % Self::APERTURE;
            let start_up = j_up % Self::APERTURE;
            let num_hops = (bary.0 * self.denominator as f64 / Self::APERTURE as f64).floor() as u32; // integer division
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
        //        let bary = IVEA3HBary::project(point);
        let projection = Vgc;
        let icosahedron = new(Orientation::DGGS_OPTIMAL);
        let projected = projection.geo_to_cartesian(vec![point], Some(&icosahedron), None);
        let face : i32 = projected[0].face.try_into().unwrap();
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

        println!("Barycentric: {:?}", bary);

        let zone_centre = IVEA3HBary::find_nearest_zone_centre(self, bary);

        let unique = IVEA3HBary::edge_cases(self, zone_centre.0, zone_centre.1, face);
        println!("After edge cases: {:?}", unique);

        // Bundle index into 64 bit
        return IVEA3HBary::bundle_zone_id(self, unique.0, unique.1, unique.2);
    }
}

#[cfg(test)]
mod tests {

    use crate::impls::ivea3h_bary::IVEA3HBary;
    use geoplegma::types::RefinementLevel;

    #[test]
    fn test_zone_from_point() {
        // ToDo    
    }

    #[test]
    fn test_find_nearest_zone_centre() {
        
        let mut system = IVEA3HBary::new(RefinementLevel::new(3).unwrap());
        let bary1 = (0.21 as f64, 0.64 as f64);
        let bary2 = (0.45 as f64, 0.22 as f64);


        let mut centre = system.find_nearest_zone_centre(bary1);
        assert_eq!(centre.0, 2);
        assert_eq!(centre.1, 5);

        centre = system.find_nearest_zone_centre(bary2);
        assert_eq!(centre.0, 5); // ==> Verify this one !!!!
        assert_eq!(centre.1, 2);
        
        system.set_refinement_level(RefinementLevel::new(4).unwrap());

        centre = system.find_nearest_zone_centre(bary1);
        assert_eq!(centre.0, 2);
        assert_eq!(centre.1, 6);

        centre = system.find_nearest_zone_centre(bary2);
        assert_eq!(centre.0, 4);
        assert_eq!(centre.1, 2);
        
        system.set_refinement_level(RefinementLevel::new(5).unwrap());

        centre = system.find_nearest_zone_centre(bary1);
        assert_eq!(centre.0, 5);
        assert_eq!(centre.1, 17);

        centre = system.find_nearest_zone_centre(bary2);
        assert_eq!(centre.0, 12);
        assert_eq!(centre.1, 6);
    }

    #[test]
    fn test_edge_cases() {
        
        let mut system = IVEA3HBary::new(RefinementLevel::new(3).unwrap());

        let mut i = 2;
        let mut j = 5;
        let mut face = 7;
        let mut unique = system.edge_cases(i, j, face);
        assert_eq!(i, unique.0);
        assert_eq!(j, unique.1);
        assert_eq!(face, unique.2);
        
        i = 0;
        j = 3;
        face = 1;
        unique = system.edge_cases(i, j, face);
        assert_eq!(i, unique.0);
        assert_eq!(j, unique.1);
        assert_eq!(face, unique.2);
        
        i = 3;
        j = 6;
        face = 9;
        unique = system.edge_cases(i, j, face);
        assert_eq!(0, unique.0);
        assert_eq!(6, unique.1);
        assert_eq!(1, unique.2);

        system = IVEA3HBary::new(RefinementLevel::new(4).unwrap());
        
        i = 3;
        j = 6;
        face = 9;
        unique = system.edge_cases(i, j, face);
        assert_eq!(0, unique.0);
        assert_eq!(6, unique.1);
        assert_eq!(1, unique.2);
        
        i = 4;
        j = 5;
        face = 13;
        unique = system.edge_cases(i, j, face);
        assert_eq!(j, unique.0);
        assert_eq!(i, unique.1);
        assert_eq!(6, unique.2);
        
        i = 0;
        j = 9;
        face = 16;
        unique = system.edge_cases(i, j, face);
        assert_eq!(i, unique.0);
        assert_eq!(j, unique.1);
        assert_eq!(12, unique.2);
        
        i = 0;
        j = 9;
        face = 7;
        unique = system.edge_cases(i, j, face);
        assert_eq!(i, unique.0);
        assert_eq!(j, unique.1);
        assert_eq!(1, unique.2);
    }
}
