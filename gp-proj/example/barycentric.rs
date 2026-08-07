// Copyright 2025 contributors to the GeoPlegma project.
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geoplegma::types::Point;
use gp_proj::projections::projections::{traits::Projection, vgc::Vgc};

pub fn main() -> () {
    println!(
        "Barycentric example for gp-proj. Geographic coordinates straight to barycentric \
         face coordinates in one call."
    );

    let points: Vec<Point> = vec![
        Point::new(38.695125, -9.222154),
        Point::new(47.7022, -138.97503),
        Point::new(25.82577, 99.72721),
        Point::new(12.89276, -64.10552),
    ];

    // Defaults: WGS84 ellipsoid, DGGS-optimal icosahedron orientation.
    let result = Vgc::default().geo_to_barycentric(points, None, None, None);

    for r in &result {
        println!(
            "face={:>2}  (i, j, k) = ({:.4}, {:.4}, {:.4})  sum={:.6}",
            r.face,
            r.coords.x,
            r.coords.y,
            r.coords.z,
            r.coords.x + r.coords.y + r.coords.z
        );
    }
}
