// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geo::{Point, Polygon};
use std::fmt;

#[derive(Debug)]
pub struct Zone {
    pub id: ZoneID,
    pub region: Polygon,
    pub center: Point,
    pub vertex_count: u32,
    pub children: Option<Vec<String>>,
    pub neighbors: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Zones {
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone)]
pub struct ZoneID {
    pub id: String,
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

impl fmt::Display for ZoneID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
