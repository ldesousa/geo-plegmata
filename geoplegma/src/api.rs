// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::DggrsError;
use crate::types::{BoundingBox, Point, RefinementLevel, RelativeDepth, ZoneId, Zones};

/// Addresses all the configuration options that apply to all port functions
///
/// Boolean switches are all set to true via the default implementation
///
/// The following output can be controlled:
/// - region geometry
/// - centroid geometry
/// - vertex_count (the number of edges/nodes
/// - children (list of ZoneIds)
/// - neighbors (list of ZoneIds)
/// - area_sqm (the area in squaremeter as calculated by `geo`'s geodesic_area_unsigned() function
/// - densify (region geometry densification)
///
#[derive(Debug, Copy, Clone)]
pub struct DggrsApiConfig {
    pub region: bool,
    pub center: bool,
    pub vertex_count: bool,
    pub children: bool,
    pub neighbors: bool,
    pub area_sqm: bool,
    pub densify: bool, // TODO:: this is the switch to generate densified gemetry, which is actually not needed for H3 due to the Gnomic projection.
}

impl Default for DggrsApiConfig {
    fn default() -> Self {
        Self {
            region: true,
            center: true,
            vertex_count: true,
            children: true,
            neighbors: true,
            area_sqm: true,
            densify: true,
        }
    }
}

/// The DGGRS port trait. Each adapter can only implement the functions defined here.
pub trait DggrsApi: Send + Sync {
    /// Get zones in the bounding box. If no bbox is supplied the whole world is taken.
    fn zones_from_bbox(
        &self,
        refinement_level: RefinementLevel,
        bbox: Option<BoundingBox>,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError>;

    /// Get zones for a Point.
    fn zone_from_point(
        &self,
        refinement_level: RefinementLevel,
        point: Point, // NOTE:Consider accepting a vector of Points.
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError>;

    /// Get zones based on a parent ZoneID.
    fn zones_from_parent(
        &self,
        relative_depth: RelativeDepth,
        parent_zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError>;

    /// Get the primary parent zone for a given ZoneID.
    ///
    /// The zone returned by this function is exactly one refinement level above the input zone. Which zone gets returned as the primary parent is dependent on the DGGRS implementation.
    fn primary_parent_from_zone(
        &self,
        zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError>;

    /// Get a zone based on a ZoneID
    fn zone_from_id(
        &self,
        zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError>; // NOTE: Consider accepting a vector of ZoneIDs

    /// Get the total number of zones at a refinement level.
    fn zone_count(&self, refinement_level: RefinementLevel) -> Result<u64, DggrsError>; // TODO: Consider hard coding zone count statistics instead of calculating them on the fly

    /// Get the minimum refinement level of a DGGRS
    fn min_refinement_level(&self) -> Result<RefinementLevel, DggrsError>;

    /// Get the maximum refinment level of a DGGRS
    fn max_refinement_level(&self) -> Result<RefinementLevel, DggrsError>;

    /// Get the default refinement level of a DGGRS
    fn default_refinement_level(&self) -> Result<RefinementLevel, DggrsError>;

    /// Get the  max relative depth of a DGGRS
    fn max_relative_depth(&self) -> Result<RelativeDepth, DggrsError>;

    /// Get the  default relative depth of a DGGRS
    fn default_relative_depth(&self) -> Result<RelativeDepth, DggrsError>;
}
