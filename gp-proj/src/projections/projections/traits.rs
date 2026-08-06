// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{
    Vector3D,
    ellipsoid::{AuthalicCoord, Ellipsoid},
    projections::{
        layout::traits::Layout,
        polyhedron::{Orientation, Polyhedron},
    },
};
use geo::{Coord, Point};

/// Barycentric coordinates of a point within a polyhedron face.
///
/// `coords.x`, `coords.y`, `coords.z` are the weights of the face's own 3 vertices (0, 1, 2
/// in whatever order the projection defines them), so `coords.x + coords.y + coords.z ==
/// 1.0` and the point is inside the face iff all three are in `[0, 1]`.
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
        ellipsoid: &dyn Ellipsoid,
    ) -> DistortionMetrics;

    fn to_3d(lat: f64, lon: f64) -> [f64; 3] {
        let x = lat.cos() * lon.cos();
        let y = lat.cos() * lon.sin();
        let z = lat.sin();

        [x, y, z]
    }

    /// Geographic coordinates straight to barycentric face coordinates, in one call.
    ///
    /// Named to mirror [`geo_to_cartesian`](Self::geo_to_cartesian): both go from a "geo"
    /// input to a projected output. Defaults to the WGS84 ellipsoid and, when no
    /// `polyhedron` is supplied, an icosahedron at [`Orientation::DGGS_OPTIMAL`]. Pass a
    /// pre-built `polyhedron` to use a different orientation/shape or to reuse one across
    /// many calls instead of rebuilding it each time. See [`ForwardBary`] for what `coords`
    /// means.
    fn geo_to_barycentric(
        &self,
        points: Vec<Point>,
        polyhedron: Option<&Polyhedron>,
        orientation: Option<Orientation>,
        ellipsoid: Option<&dyn Ellipsoid>,
    ) -> Vec<ForwardBary>;
}
