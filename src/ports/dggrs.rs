// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::models::common::CellsGEO;
use geo::Point;
// That is the port
pub trait DggrsPort: Send + Sync {
    fn whole_earth(
        &self,
        dggs_res_spec: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> CellsGEO;
    fn from_point(&self, dggs_res_spec: u8, point: Point, densify: bool) -> CellsGEO;
    fn coarse_cells(
        &self,
        dggs_res_spec: u8,
        clip_cell_addresses: String,
        // clip_cell_res: u8,
        densify: bool,
    ) -> CellsGEO;
    fn single_zone(&self, zone_id: String, densify: bool) -> CellsGEO;
}
