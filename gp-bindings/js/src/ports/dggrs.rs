// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use std::{str::FromStr, sync::Arc};

use geo::{Coord, Rect};
use geoplegma::{
  api::{DggrsApi, DggrsApiConfig},
  factory,
  types::{BoundingBox, DggrsUid, HexString, Point, RefinementLevel, RelativeDepth, ZoneId},
};
use napi::{Either, Error};

use crate::models::common::{JsZones, ZonesWrapper};

use napi_derive::napi;

#[napi]
pub struct Dggrs {
  inner: Arc<dyn DggrsApi>,
}

#[napi(object)]
pub struct Config {
  pub region: bool,
  pub center: bool,
  pub vertex_count: bool,
  pub children: bool,
  pub neighbors: bool,
  pub area_sqm: bool,
  pub densify: bool, // TODO:: this is the switch to generate densified gemetry, which is actually not needed for H3 due to the Gnomic projection.
}

#[napi]
impl Default for Config {
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

#[napi]
pub fn default_config() -> Config {
  Config {
    region: true,
    center: true,
    vertex_count: true,
    children: true,
    neighbors: true,
    area_sqm: true,
    densify: true,
  }
}

#[napi]
impl Dggrs {
  #[napi(constructor)]
  pub fn new(dggrs: String) -> Dggrs {
    let dggrs_uid = DggrsUid::from_str(&dggrs).expect("Invalid DGGRS UID");

    Dggrs {
      inner: factory::get(dggrs_uid).expect("msg"),
    }
  }

  #[napi(js_name = zonesFromBbox)]
  pub fn zones_from_bbox(
    &self,
    refinement_level: i32,
    bbox: Option<Vec<Vec<f64>>>,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let refinement_level_ = RefinementLevel::new(refinement_level).unwrap();

    let bbox_: Option<BoundingBox> = match bbox {
      Some(b) => Some(BoundingBox::new(b[0][0], b[0][1], b[1][0], b[1][1])),
      _ => None,
    };

    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsApiConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zones_from_bbox(refinement_level_, bbox_, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    Ok(zones.to_export())
  }

  #[napi(js_name = zoneFromPoint)]
  pub fn zone_from_point(
    &self,
    refinement_level: i32,
    point: Option<Vec<f64>>,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let refinement_level_ = RefinementLevel::new(refinement_level).unwrap();
    let point_ = point.unwrap();
    let geo_pt = Point::new(point_[0], point_[1]);

    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsApiConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zone_from_point(refinement_level_, geo_pt, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };
    Ok(zones.to_export())
  }

  #[napi(js_name = zonesFromParent)]
  pub fn zones_from_parent(
    &self,
    relative_depth: i32,
    parent_zone_id: Either<String, i64>,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let relative_depth_ = RelativeDepth::new(relative_depth).unwrap();
    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsApiConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };

    let parent_zone_id_ = match parent_zone_id {
      Either::B(num) => ZoneId::IntId(num.try_into().unwrap()),

      Either::A(s) => {
        if is_zone_hex_id(&s) {
          ZoneId::HexId(HexString::new(&s).unwrap())
        } else {
          ZoneId::StrId(s)
        }
      }
    };

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zones_from_parent(relative_depth_, parent_zone_id_, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    Ok(zones.to_export())
  }

  #[napi(js_name = zoneFromId)]
  pub fn zone_from_id(
    &self,
    zone_id: Either<String, i64>,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsApiConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };

    let zone_id_ = match zone_id {
      Either::B(num) => ZoneId::IntId(num.try_into().unwrap()),

      Either::A(s) => {
        if is_zone_hex_id(&s) {
          ZoneId::HexId(HexString::new(&s).unwrap())
        } else {
          ZoneId::StrId(s)
        }
      }
    };

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zone_from_id(zone_id_, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    Ok(zones.to_export())
  }

  #[napi(js_name = zoneCount)]
  pub fn zone_count(&self, refinement_level: i32) -> napi::Result<u32> {
    let refinement_level_ = RefinementLevel::new(refinement_level).unwrap();

    let count = self
      .inner
      .zone_count(refinement_level_)
      .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(count.try_into().unwrap())
  }

  #[napi(js_name = minRefinementLevel)]
  pub fn min_refinement_level(&self) -> napi::Result<i32> {
    let rl = self
      .inner
      .min_refinement_level()
      .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(rl.get())
  }

  #[napi(js_name = maxRefinementLevel)]
  pub fn max_refinement_level(&self) -> napi::Result<i32> {
    let rl = self
      .inner
      .max_refinement_level()
      .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(rl.get())
  }

  #[napi(js_name = defaulRefinementLevel)]
  pub fn default_refinement_level(&self) -> napi::Result<i32> {
    let rl = self
      .inner
      .default_refinement_level()
      .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(rl.get())
  }

  #[napi(js_name = maxRelativeDepth)]
  pub fn max_relative_depth(&self) -> napi::Result<i32> {
    let rl = self
      .inner
      .max_relative_depth()
      .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(rl.get())
  }

  #[napi(js_name = defaultRelativeDepth)]
  pub fn default_relative_depth(&self) -> napi::Result<i32> {
    let rl = self
      .inner
      .default_relative_depth()
      .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(rl.get())
  }
}

fn is_zone_hex_id(s: &str) -> bool {
  s.len() == 16 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_zone() {
    let generator = Dggrs::new("ISEA3HDGGAL".to_owned());
    let rl = RefinementLevel::new(1).unwrap();
    let bbox = BoundingBox::new(-77.0, 39.0, -76.0, 40.0);
    let result = generator
      .zones_from_bbox(
        1,
        Some(vec![vec![-77.0, 39.0], vec![-76.0, 40.0]]),
        Some(Config {
          region: true,
          center: true,
          vertex_count: true,
          children: true,
          neighbors: true,
          area_sqm: true,
          densify: false,
        }),
      )
      .unwrap();

    println!("{:?}", result);
    // assert_eq!(
    //   result.zones.len(),
    //   1,
    //   "{:?}: zones_from_bbox returned wrong result",
    //   result.zones
    // );
  }
}
