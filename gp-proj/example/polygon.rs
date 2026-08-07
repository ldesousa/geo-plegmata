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
    constants::WGS84,
    ellipsoid::AuthalicSphere,
    projections::{
        polyhedron::{icosahedron, Orientation},
        projections::{traits::Projection, vgc::Vgc},
    },
};

pub fn main() -> () {
    println!(
        "Polygon example for gp-proj. Convert geographic coordinates to barycentric coordinates, and vice-versa."
    );

    let p1 = Point::new(38.72423364219293, -9.192722996293583);
    let p2 = Point::new(37.83692529759742, -10.681508330872333);
    let p3 = Point::new(36.23040220266431, -9.027302403562487);
    let p4 = Point::new(37.48772339897228, -6.049731734403736);
    let p5 = Point::new(39.57941279302861, -7.180105784733257);
    let p6 = Point::new(38.72423364219293, -9.192722996293583);

    let projection = Vgc::default();
    let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);
    let sphere = AuthalicSphere::from_ellipsoid(&WGS84);
    let points = vec![p1, p2, p3, p4, p5, p6]
        .into_iter()
        .map(|p| sphere.convert(p))
        .collect();
    let coords = projection.geo_to_cartesian(points, Some(&icosahedron), None);

    println!("{:?}", coords);
}
