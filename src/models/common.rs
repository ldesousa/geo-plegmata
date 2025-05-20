// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

#[derive(Debug)]
pub struct PositionGeo {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

impl Position2D {
    pub fn from_tuple(t: (u8, u8)) -> Position2D {
        Position2D { x: f64::from(t.0), y: f64::from(t.1) }
    }
    pub fn mid(a: Position2D, b: Position2D) -> Position2D {
        Position2D {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        }
    }
}
