// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use super::vector_3d::Vector3D;

#[derive(Debug)]
pub struct Plane {
    a: f64,
    b: f64,
    c: f64,
}

impl Plane {
    pub fn from_points(p0: Vector3D, p1: Vector3D, p2: Vector3D) -> Self {
        let u = p1.subtract(p0);
        let v = p2.subtract(p0);
        let n = u.cross(v).normalize();
        Self {
            a: n.x,
            b: n.y,
            c: n.z,
        }
    }

    pub fn signed_distance(&self, point: Vector3D) -> f64 {
        self.a * point.x + self.b * point.y + self.c * point.z
    }
}