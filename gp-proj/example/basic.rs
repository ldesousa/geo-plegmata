// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
// Modified by João Manuel (joao.manuel@geoinsight.ai)
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geo::Point;
use gp_proj::projections::{
    polyhedron::icosahedron::new,
    projections::{traits::Projection, vgc::Vgc},
};

pub fn main() -> () {
    println!(
        "Basic example for gp-proj. Convert geographic coordinates to barycentric coordinates, and vice-versa."
    );

    let points = [
        Point::new(-9.222154, 38.695125),
        Point::new(-138.97503, 47.7022),
        Point::new(99.72721, 25.82577),
        Point::new(-64.10552, 12.89276),
        Point::new(-128.28185, -50.60992),
        Point::new(-70.47681, -0.81784),
        Point::new(152.44705, -21.59114),
        Point::new(66.665798, -77.717034),
        Point::new(63.501735, 80.099071),
        Point::new(0.0, 45.0),
        Point::new(30.0, 30.0),
    ];

    let projection = Vgc;
    let icosahedron = new();

    let coords = projection.geo_to_cartesian(points.to_vec(), Some(&icosahedron), None);

    for (i, p) in coords.iter().enumerate() {
        println!(
            "({:>10.4}, {:>10.4}) -> face={} x={:.4} y={:.4}",
            points[i].x(),
            points[i].y(),
            p.face,
            p.coords.x,
            p.coords.y
        );
    }

    let distortion = projection.compute_distortion(38.68499, -9.49420, &icosahedron);
    println!("h: {} (expected: 0.7580403)", distortion.h);
    println!("k: {} (expected: 1.333174)", distortion.k);
    println!(
        "Angular deformation: {}° (expected: 33.045°)",
        distortion.angular_deformation
    );
    println!("Areal scale: {} (expected: ~1.0)", distortion.areal_scale);
}
