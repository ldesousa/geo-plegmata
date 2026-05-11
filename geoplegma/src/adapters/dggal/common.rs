// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::api::DggrsApiConfig;
use crate::error::DggrsError;
use crate::error::dggal::DggalError;
use crate::types::{BoundingBox, Point, Region, Zone, ZoneId, Zones};
use dggal_rust::dggal::{DGGRS, DGGRSZone, GeoExtent, GeoPoint};
use geo::GeodesicArea;

pub fn to_zones(
    dggrs: DGGRS,
    dggal_zones: Vec<DGGRSZone>,
    conf: DggrsApiConfig,
) -> Result<Zones, DggalError> {
    let zones: Vec<Zone> = dggal_zones
        .into_iter()
        .map(|dggal_zone| {
            let txt = dggrs.getZoneTextID(dggal_zone);

            let id_string = ZoneId::new_str(&txt)
                .map_err(|e: DggrsError| DggalError::InvalidZoneIdFormat(format!("{txt} ({e})")))?;

            let center = if conf.center {
                let center_point = dggrs.getZoneWGS84Centroid(dggal_zone);
                Some(to_point(&center_point))
            } else {
                None
            };

            let region = if conf.region || conf.area_sqm {
                let dggal_geo_points = if conf.densify {
                    dggrs.getZoneRefinedWGS84Vertices(dggal_zone, 0)
                } else {
                    dggrs.getZoneWGS84Vertices(dggal_zone)
                };
                Some(to_polygon(&dggal_geo_points))
            } else {
                None
            };

            let area_sqm = if conf.area_sqm {
                region.as_ref().map(|r| r.to_geo_polygon().geodesic_area_unsigned())
            } else {
                None
            };

            let vertex_count = if conf.vertex_count {
                let vc = dggrs.countZoneEdges(dggal_zone).try_into().map_err(|e| {
                    DggalError::EdgeCountConversion {
                        zone_id: id_string.to_string(),
                        source: e,
                    }
                })?;
                Some(vc)
            } else {
                None
            };

            let children = if conf.children {
                Some(
                    dggrs
                        .getZoneChildren(dggal_zone)
                        .into_iter()
                        .map(|z| to_str_zone_id(&dggrs, z))
                        .collect::<Result<Vec<_>, DggalError>>()?,
                )
            } else {
                None
            };

            let neighbors = if conf.neighbors {
                let mut nb_types: [i32; 6] = [0; 6]; // WARN: don't replace this
                Some(
                    dggrs
                        .getZoneNeighbors(dggal_zone, &mut nb_types)
                        .into_iter()
                        .map(|n| to_str_zone_id(&dggrs, n))
                        .collect::<Result<Vec<_>, DggalError>>()?,
                )
            } else {
                None
            };

            Ok(Zone {
                id: id_string,
                region,
                center,
                vertex_count,
                children,
                neighbors,
                area_sqm,
            })
        })
        .collect::<Result<Vec<Zone>, DggalError>>()?;

    Ok(Zones { zones })
}

fn to_point(pt: &GeoPoint) -> Point {
    Point::new(pt.lat.to_degrees(), pt.lon.to_degrees())
}

fn to_polygon(points: &[GeoPoint]) -> Region {
    let mut pts: Vec<_> = points
        .iter()
        .map(|pt| Point::new(pt.lat.to_degrees(), pt.lon.to_degrees()))
        .collect();

    if let Some(first) = pts.first().copied() {
        if pts.last().copied() != Some(first) {
            pts.push(first);
        }
    }

    Region::new(pts)
}

fn to_u64_zone_id(id: DGGRSZone) -> ZoneId {
    // NOTE: Expand this to do the conversion automatically
    ZoneId::IntId(id)
}

fn to_str_zone_id(dggrs: &DGGRS, zone: DGGRSZone) -> Result<ZoneId, DggalError> {
    let txt = dggrs.getZoneTextID(zone);
    ZoneId::new_str(&txt)
        .map_err(|e: DggrsError| DggalError::InvalidZoneIdFormat(format!("{txt} ({e})")))
}

pub fn to_geo_point(pt: Point) -> GeoPoint {
    GeoPoint {
        lat: pt.lat.to_radians(),
        lon: pt.lon.to_radians(),
    }
}

pub fn bbox_to_geoextent(bbox: &BoundingBox) -> GeoExtent {
    GeoExtent {
        ll: GeoPoint {
            lat: bbox.min_lat.to_radians(),
            lon: bbox.min_lon.to_radians(),
        },
        ur: GeoPoint {
            lat: bbox.max_lat.to_radians(),
            lon: bbox.max_lon.to_radians(),
        },
    }
}
