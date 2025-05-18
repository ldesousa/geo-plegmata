// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::models::common::Zones;
use geo::Point;
// That is the port
pub trait DggrsPort: Send + Sync {
    fn zones_from_bbox(&self, depth: u8, densify: bool, bbox: Option<Vec<Vec<f64>>>) -> Zones;
    fn zone_from_point(&self, depth: u8, point: Point, densify: bool) -> Zones;
    fn zones_from_parent(
        &self,
        depth: u8,
        parent_zone_id: String,
        // clip_cell_res: u8,
        densify: bool,
    ) -> Zones;
    fn zone_from_id(&self, zone_id: String, densify: bool) -> Zones;
}
