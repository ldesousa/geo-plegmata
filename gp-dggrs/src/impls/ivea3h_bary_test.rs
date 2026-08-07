// Copyright 2026 contributors to the GeoPlegma project.
// Originally authored by Luís Moreira de Sousa, Técnico, ULisboa
// (luis.moreira.de.sousa [at] tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::impls::ivea3h_bary::IVEA3HBary;
use base64::{Engine as _, engine::general_purpose};
use geoplegma::types::RefinementLevel;

#[cfg(test)]
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
fn test_bundle_zone_id() {
    //    let mut system = IVEA3HBary::new(RefinementLevel::new(3).unwrap());
    //    let mut i = 2;
    //    let mut j = 5;
    //    let mut face = 7;
    //
    //    assert_eq!(
    //        base64::encode(hex::decode("0x670000014000002").unwrap()),
    //        system.bundle_zone_id(i, j, face)
    //    );
}

#[test]
fn test_unbundle_zone_id() {
    //    let mut system = IVEA3HBary::new(RefinementLevel::new(3).unwrap());
    //
    //    let (i, j, face, level) =
    //        system.unbundle_zone_id(base64::encode(hex::decode("0x670000014000002").unwrap()));
    //    assert_eq!(i, 2);
    //    assert_eq!(j, 5);
    //    assert_eq!(face, 7);
    //    assert_eq!(level, 3);
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
