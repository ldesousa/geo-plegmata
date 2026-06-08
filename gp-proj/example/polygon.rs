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
    polyhedron::{icosahedron, Orientation},
    projections::{traits::Projection, vgc::Vgc},
};

pub fn main() -> () {
    println!(
        "Polygon example for gp-proj. Convert geographic coordinates to barycentric coordinates, and vice-versa."
    );

    let p1 = Point::new(-9.192722996293583, 38.72423364219293);
    let p2 = Point::new(-10.681508330872333, 37.83692529759742);
    let p3 = Point::new(-9.027302403562487, 36.23040220266431);
    let p4 = Point::new(-6.049731734403736, 37.48772339897228);
    let p5 = Point::new(-7.180105784733257, 39.57941279302861);
    let p6 = Point::new(-9.192722996293583, 38.72423364219293);

    let projection = Vgc;
    let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);
    let coords = projection.geo_to_face(vec![p1, p2, p3, p4, p5, p6], Some(&icosahedron));

    println!("{:?}", coords);
}
