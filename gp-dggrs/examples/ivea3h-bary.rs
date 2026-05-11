// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Luís Moreira de Sousa, Técnico, ULisboa 
// (luis.moreira.de.sousa [at] tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geo::Point;
use geoplegma::types::RefinementLevel;
use gp_dggrs::sys_api::DggrsSysApi;
use gp_dggrs::impls::ivea3h_bary::IVEA3HBary;

fn unbundle_index(zone_id:u64) -> (u64, u64, u64, u64) {

    let bary_i = zone_id % 2_u64.pow(26);
    let mut tail:u64 = zone_id / 2_u64.pow(26);
    let bary_j = tail % 2_u64.pow(26); 
    tail = tail / 2_u64.pow(26);
    let face = tail % 2_u64.pow(5);
    let level = tail / 2_u64.pow(5);

    return (bary_i, bary_j, face, level);
}

/// This is just an example and basic testing function if there is output or not
pub fn main() {

    // Old points
    //let p1 = Point::new(0.45, 0.22); // Longitude: 43, Latitude: 49.5
    //let p2 = Point::new(0.21, 0.64); // Longitude: 40, Latitude: 71
    //let p3 = Point::new(0.75, 0.11);
    let p1 = Point::new(43.0, 49.5);
    let p2 = Point::new(40.0, 71.0);

    let system = IVEA3HBary {};
    let level = RefinementLevel::new(3).unwrap();

    println!("== Resolution 3 ==");
    println!("Point 1 {} {}", p1.x(), p1.y());
    let zone1 = system.zone_from_point(level, p1);
    let mut unbundled = unbundle_index(zone1);
    assert_eq!(unbundled.0, 5);
    assert_eq!(unbundled.1, 2);   
    assert_eq!(unbundled.2, 10);
    assert_eq!(unbundled.3, 3);   


    println!("Point 2 {} {}", p2.x(), p2.y());
    let zone2 = system.zone_from_point(level, p2);
    unbundled = unbundle_index(zone2);
    assert_eq!(unbundled.0, 2);
    assert_eq!(unbundled.1, 5);   
    assert_eq!(unbundled.2, 10);
    assert_eq!(unbundled.3, 3);   
    
    let level = RefinementLevel::new(4).unwrap();

    println!("\n== Resolution 4 ==");
    println!("Point 1 {} {}", p1.x(), p1.y());
    let zone3 = system.zone_from_point(level, p1);
    unbundled = unbundle_index(zone3);
    assert_eq!(unbundled.0, 4);
    assert_eq!(unbundled.1, 2);   
    assert_eq!(unbundled.2, 10);
    assert_eq!(unbundled.3, 4);   

    println!("Point 2 {} {}", p2.x(), p2.y());
    let zone4 = system.zone_from_point(level, p2);
    unbundled = unbundle_index(zone4);
    assert_eq!(unbundled.0, 2);
    assert_eq!(unbundled.1, 6);   
    assert_eq!(unbundled.2, 10);
    assert_eq!(unbundled.3, 4);   
    
    let level = RefinementLevel::new(5).unwrap();

    println!("\n== Resolution 5 ==");
    println!("Point 1 {} {}", p1.x(), p1.y());
    let zone3 = system.zone_from_point(level, p1);
    unbundled = unbundle_index(zone3);
    assert_eq!(unbundled.0, 12);
    assert_eq!(unbundled.1, 6);   
    assert_eq!(unbundled.2, 10);
    assert_eq!(unbundled.3, 5);   

    println!("Point 2 {} {}", p2.x(), p2.y());
    let zone4 = system.zone_from_point(level, p2);
    unbundled = unbundle_index(zone4);
    assert_eq!(unbundled.0, 5);
    assert_eq!(unbundled.1, 17);   
    assert_eq!(unbundled.2, 10);
    assert_eq!(unbundled.3, 5);   
    
    let level = RefinementLevel::new(6).unwrap();

    println!("\n== Resolution 6 ==");
    println!("Point 1 {} {}", p1.x(), p1.y());
    let zone3 = system.zone_from_point(level, p1);
    unbundled = unbundle_index(zone3);
    assert_eq!(unbundled.0, 12);
    assert_eq!(unbundled.1, 6);   
    assert_eq!(unbundled.2, 10);
    assert_eq!(unbundled.3, 6);   

    println!("Point 2 {} {}", p2.x(), p2.y());
    let zone4 = system.zone_from_point(level, p2);
    unbundled = unbundle_index(zone4);
    assert_eq!(unbundled.0, 6);
    assert_eq!(unbundled.1, 17);   
    assert_eq!(unbundled.2, 10);
    assert_eq!(unbundled.3, 6);   

    //println!("Point 3 {} {}", p3.x(), p3.y());
    //let zone5 = system.zone_from_point(level, p3);
    //unbundled = unbundle_index(zone5);
    //assert_eq!(unbundled.0, 20);
    //assert_eq!(unbundled.1, 3);   
    //assert_eq!(unbundled.2, 10);
    //assert_eq!(unbundled.3, 6);   
}
