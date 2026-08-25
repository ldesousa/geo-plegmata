// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Luís Moreira de Sousa, Técnico, ULisboa
// (luis.moreira.de.sousa [at] tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geoplegma::types::{Point, RefinementLevel};
use gp_dggrs::impls::ivea3h_bary::IVEA3HBary;
use gp_dggrs::sys_api::DggrsSysApi;

/// This is just an example and basic testing function if there is output or not
pub fn main() {
    // Old points
    //let p1 = Point::new(0.45, 0.22); // Longitude: 43, Latitude: 49.5
    //let p2 = Point::new(0.21, 0.64); // Longitude: 40, Latitude: 71
    //let p3 = Point::new(0.75, 0.11);
    //

    let p1 = Point::new(43.0, 49.5);
    let p2 = Point::new(40.0, 71.0);
    // Edge cases
    let p3 = Point::new(0.0, 45.0);
    let p4 = Point::new(30.0, 30.0);
    let p5 = Point::new(-9.49420, 38.68499);

    let level = RefinementLevel::new(3).unwrap();
    let system = IVEA3HBary::new(level);

    println!("== Resolution 3 ==");
    println!("Point 1 {} {}", p1.lat, p1.lon);
    let zone1 = system.zone_from_point(level, p1);
    let mut unbundled = IVEA3HBary::unbundle_zone_id(zone1);
    println!(
        "Unbundled: i:{} j:{} Face:{} Level:{}",
        unbundled.0, unbundled.1, unbundled.2, unbundled.3
    );
    assert_eq!(unbundled.0, 4);
    assert_eq!(unbundled.1, 4);
    assert_eq!(unbundled.2, 3);
    assert_eq!(unbundled.3, RefinementLevel::new(3).expect("REASON"));

    println!("Point 2 {} {}", p2.lat, p2.lon);
    let zone2 = system.zone_from_point(level, p2);
    unbundled = IVEA3HBary::unbundle_zone_id(zone2);
    println!(
        "Unbundled: i:{} j:{} Face:{} Level:{}",
        unbundled.0, unbundled.1, unbundled.2, unbundled.3
    );
    assert_eq!(unbundled.0, 4);
    assert_eq!(unbundled.1, 4);
    assert_eq!(unbundled.2, 5);
    assert_eq!(unbundled.3, RefinementLevel::new(3).expect("REASON"));

    println!("Point 3 {} {}", p3.lat, p3.lon);
    let zone3 = system.zone_from_point(level, p3);
    unbundled = IVEA3HBary::unbundle_zone_id(zone3);
    println!(
        "Unbundled: i:{} j:{} Face:{} Level:{}",
        unbundled.0, unbundled.1, unbundled.2, unbundled.3
    );
    assert_eq!(unbundled.0, 0);
    assert_eq!(unbundled.1, 6);
    assert_eq!(unbundled.2, 9);
    assert_eq!(unbundled.3, RefinementLevel::new(3).expect("REASON"));

    println!("Point 4 {} {}", p4.lat, p4.lon);
    let zone4 = system.zone_from_point(level, p4);
    unbundled = IVEA3HBary::unbundle_zone_id(zone4);
    println!(
        "Unbundled: i:{} j:{} Face:{} Level:{}",
        unbundled.0, unbundled.1, unbundled.2, unbundled.3
    );
    assert_eq!(unbundled.0, 1);
    assert_eq!(unbundled.1, 1);
    assert_eq!(unbundled.2, 1);
    assert_eq!(unbundled.3, RefinementLevel::new(3).expect("REASON"));

    println!("Point 5 {} {}", p5.lat, p5.lon);
    let zone5 = system.zone_from_point(level, p5);
    unbundled = IVEA3HBary::unbundle_zone_id(zone5);
    println!(
        "Unbundled: i:{} j:{} Face:{} Level:{}",
        unbundled.0, unbundled.1, unbundled.2, unbundled.3
    );
    assert_eq!(unbundled.0, 2);
    assert_eq!(unbundled.1, 2);
    assert_eq!(unbundled.2, 9);
    assert_eq!(unbundled.3, RefinementLevel::new(3).expect("REASON"));
}
