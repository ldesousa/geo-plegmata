// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

#[derive(Debug)]
pub struct PointOld {
    pub lon: f64,
    pub lat: f64,
}
#[derive(Debug, Clone)]
pub struct ZoneID {
    pub id: String,
}

#[derive(Debug)]
pub struct ZoneCoord {
    pub lon: f64,
    pub lat: f64,
}
#[derive(Debug)]
pub struct ZoneGeom {
    pub geom: Vec<ZoneCoord>,
}
#[derive(Debug)]
pub struct Zone {
    pub id: ZoneID,
    pub geom: ZoneGeom,
}
#[derive(Debug)]
pub struct Zones {
    pub zones: Vec<Zone>,
}
#[derive(Debug)]
pub struct IdArray {
    pub id: Option<String>,
    pub arr: Option<Vec<String>>,
}

impl ZoneID {
    pub fn new(id: &str) -> Result<Self, String> {
        if (id.len() == 16 || id.len() == 18) && id.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(ZoneID { id: id.to_string() })
        } else {
            Err("ID must be exactly 16 or 18 alphanumeric characters.".to_string())
        }
    }
}

impl Default for ZoneID {
    fn default() -> Self {
        ZoneID {
            id: "0000000000000000".to_string(),
        } // Some valid default ID
    }
}

impl Default for ZoneGeom {
    fn default() -> Self {
        ZoneGeom { geom: Vec::new() }
    }
}

impl fmt::Display for ZoneID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl fmt::Display for ZoneCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Longitude: {:.6}, Latitude: {:.6}", self.lon, self.lat)
    }
}
