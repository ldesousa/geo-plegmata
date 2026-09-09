// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
// Modified by João Manuel (joao.manuel@geoinsight.ai)
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geoplegma::types::Point;
use gp_proj::{
    constants::WGS84, ellipsoid::AuthalicSphere, projections::{
        polyhedron::{Orientation, icosahedron},
        projections::{traits::Projection, vgc::Vgc},
    },
};

pub fn main() -> () {
    println!(
        "Basic example for gp-proj. Convert geographic coordinates to barycentric coordinates, and vice-versa."
    );

    let points: Vec<Point> = vec![
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

    let projection = Vgc::default();
    let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);

    let sphere = AuthalicSphere::from_ellipsoid(&WGS84);
    let authalic_points = points.iter().map(|p| sphere.to_authalic(*p)).collect();
    let forward = projection.geo_to_cartesian(authalic_points, Some(&icosahedron), None);

    let inverse = projection.cartesian_to_geo(forward, Some(&icosahedron));

    println!("original:  lat={}, lon={}", points[0].lat, points[0].lon);
    println!(
        "recovered: lat={:.6}, lon={:.6}",
        inverse[0].lat,
        inverse[0].lon
    );
}
