// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
// Modified by Sunayana Ghosh (sunayanag@gmail.com)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms


#[derive(Debug, Clone)]
pub enum Face {
    Triangle([usize; 3]),
    Quad([usize; 4]),
    Pentagon([usize; 5]),
    Hexagon([usize; 6]),
    Polygon(Vec<usize>), // for rare or irregular faces
}

impl Face {
    pub fn indices(&self) -> &[usize] {
        match self {
            Face::Triangle(v) => v,
            Face::Quad(v) => v,
            Face::Pentagon(v) => v,
            Face::Hexagon(v) => v,
            Face::Polygon(v) => v,
        }
    }
}

#[derive(Default)]
pub struct ArcLengths {
    pub ab: f64,
    pub bc: f64,
    pub ac: f64,
    pub ap: f64,
    pub bp: f64,
    pub cp: f64,
}

/// Orientation of a polyhedron on the unit sphere, expressed as the geographic
/// position (in degrees) where vertex 0 is placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orientation {
    pub lat_deg: f64,
    pub lon_deg: f64,
}

impl Orientation {
    pub fn new(lat_deg: f64, lon_deg: f64) -> Self {
        Self { lat_deg, lon_deg }
    }

    /// Vertex 0 at the geographic north pole — the canonical mathematical orientation.
    pub const POLAR: Self = Self { lat_deg: 90.0, lon_deg: 0.0 };

    /// DGGS-optimal icosahedron orientation.
    /// Places the first vertex over the ocean near Sweden (58.397145907431°N, 11.20°E)
    /// to avoid placing icosahedron singularities over major land areas.
    /// The orientation is symmetric about the equator and closely follows the ISEA orientation proposed by Sahr (2003), with a small westward shift so that only a single vertex falls on land.
    pub const DGGS_OPTIMAL: Self = Self { lat_deg: 58.397145907431, lon_deg: 11.20 };
}
