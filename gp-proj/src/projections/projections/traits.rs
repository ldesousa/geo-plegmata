// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{
    Vector3D, ellipsoid::AuthalicCoord, projections::{layout::traits::Layout, polyhedron::Polyhedron}
};
use geo::{Coord, Point};

#[derive(Debug)]
pub struct ForwardBary {
    pub coords: Vector3D,
    pub face: usize,
}

#[derive(Debug)]
pub struct ForwardCartesian {
    pub coords: Coord,
    pub face: usize,
}

#[derive(Debug)]
pub struct DistortionMetrics {
    pub h: f64,
    pub k: f64,
    pub angular_deformation: f64,
    pub areal_scale: f64,
}

pub trait Projection {
    fn geo_to_cartesian(
        &self,
        positions: Vec<AuthalicCoord>,
        polyhedron: Option<&Polyhedron>,
        layout: Option<&dyn Layout>,
    ) -> Vec<ForwardCartesian>;
    fn cartesian_to_geo(&self, coords: Vec<Coord>) -> Point;

    fn compute_distortion(
        &self,
        lat: f64,
        lon: f64,
        polyhedron: &Polyhedron,
    ) -> DistortionMetrics;

    fn to_3d(lat: f64, lon: f64) -> [f64; 3] {
        let x = lat.cos() * lon.cos();
        let y = lat.cos() * lon.sin();
        let z = lat.sin();

        [x, y, z]
    }

}
