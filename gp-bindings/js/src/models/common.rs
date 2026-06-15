// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use geo::{LineString, Polygon};
use geoplegma::types::{Point, Region, Zone, ZoneId, Zones};
use napi_derive::napi;

#[napi(object, js_name=Zone)]
pub struct RustZone {
  pub id: Id,
  pub region: Vec<Vec<f64>>,
  pub center: Vec<f64>,
  pub vertex_count: u32,
  pub children: Option<Vec<Id>>,
  pub neighbors: Option<Vec<Id>>,
}

#[napi(string_enum)]
pub enum Id {
  String,
  U64,
}

/// The Zone struct has nested heap allocations (String, Vec<(f64,f64)>, Vec<String>), which means:
/// - Each String is 24 bytes (ptr, len, capacity) + heap data.
/// - Each (f64, f64) is fine in Rust, but Vec<(f64,f64)> is not a flat Vec<f64> in wasm/napi-rs.
/// - wasm/napi-rs will have to walk and serialize everything, which is slow for thousands of zones.
/// No napi-rs overhead per zone — you pass one pointer + length per field instead of millions of small objects.
/// Zero-copy — JS reads directly from WebAssembly memory.
/// Keeps geometry-heavy Zone struct in Rust for efficient calculations.
/// Scales to millions of zones without crashing the browser or blowing up memory usage.
/// Tradeoff
/// Pro: Hugely faster for large datasets
/// Con: JS side reconstruction is manual — you need to decode UTF-8 strings from byte arrays using TextDecoder and use offset tables.
#[napi(object, js_name=FlatZones)]
#[derive(Debug)]
pub struct JsZones {
  // zone ids flattened
  pub id_offsets: Vec<u32>, // len = num_zones (start index of each id in utf8_ids)
  pub utf8_ids: Vec<u8>,

  // centers
  pub center_x: Vec<f64>,
  pub center_y: Vec<f64>,

  // vertex counts
  pub vertex_count: Vec<u32>,

  // regions (flattened coordinates)
  pub region_offsets: Vec<u32>, // len = num_zones (start index of each zone's coords in region_coords)
  pub region_coords: Vec<f64>,  // flattened x,y,x,y,...

  // children (IDs, flattened UTF-8)
  pub children_offsets: Vec<u32>, // start index into children_utf8_ids
  pub children_id_offsets: Vec<u32>, // start index of each child string inside children_utf8_ids
  pub children_utf8_ids: Vec<u8>, // raw utf8

  // neighbors (IDs, flattened UTF-8)
  pub neighbors_offsets: Vec<u32>,
  pub neighbors_id_offsets: Vec<u32>,
  pub neighbors_utf8_ids: Vec<u8>,

  pub area_sqm: Vec<f64>,
}

#[napi]
impl JsZones {
  /// Rebuild a `Zones` struct from a flattened `ZonesExport`
  pub fn to_import(&self) -> Zones {
    let zone_count = self.id_offsets.len();
    let mut zones = Vec::with_capacity(zone_count);

    let decoder = |bytes: &[u8]| String::from_utf8(bytes.to_vec()).unwrap();
    // 1) reconstruct id strings
    let mut ids: Vec<String> = Vec::with_capacity(zone_count);
    for i in 0..zone_count {
      let start = self.id_offsets[i] as usize;
      let end = if i + 1 < zone_count {
        self.id_offsets[i + 1] as usize
      } else {
        self.utf8_ids.len()
      };
      let s = str::from_utf8(&self.utf8_ids[start..end]).expect("invalid utf8 in id buffer");
      ids.push(s.to_string());
    }
    // 2) build zones
    for i in 0..zone_count {
      // region
      let region_start = self.region_offsets[i] as usize;
      let region_end = if i + 1 < zone_count {
        self.region_offsets[i + 1] as usize
      } else {
        self.region_coords.len()
      };
      let mut coords = Vec::new();
      let mut j = region_start;
      while j + 1 < region_end {
        coords.push(Point::new(self.region_coords[j], self.region_coords[j + 1]));
        j += 2;
      }
      // let line_string: geoplegma::types::Point = );
      let region = Region::new(coords);

      // children
      let c_start = self.children_offsets[i] as usize;
      let c_end = if i + 1 < self.children_offsets.len() {
        self.children_offsets[i + 1] as usize
      } else {
        self.children_id_offsets.len()
      };
      let mut children = Vec::new();
      for j in c_start..c_end {
        let s = self.children_id_offsets[j] as usize;
        let e = if j + 1 < self.children_id_offsets.len() {
          self.children_id_offsets[j + 1] as usize
        } else {
          self.children_utf8_ids.len()
        };
        let id_str = decoder(&self.children_utf8_ids[s..e]);
        children.push(ZoneId::StrId(id_str));
      }
      let children = if children.is_empty() {
        None
      } else {
        Some(children)
      };

      // neighbors
      let n_start = self.neighbors_offsets[i] as usize;
      let n_end = if i + 1 < self.neighbors_offsets.len() {
        self.neighbors_offsets[i + 1] as usize
      } else {
        self.neighbors_id_offsets.len()
      };
      let mut neighbors = Vec::new();
      for j in n_start..n_end {
        let s = self.neighbors_id_offsets[j] as usize;
        let e = if j + 1 < self.neighbors_id_offsets.len() {
          self.neighbors_id_offsets[j + 1] as usize
        } else {
          self.neighbors_utf8_ids.len()
        };
        let id_str = decoder(&self.neighbors_utf8_ids[s..e]);
        neighbors.push(ZoneId::StrId(id_str));
      }
      let neighbors = if neighbors.is_empty() {
        None
      } else {
        Some(neighbors)
      };

      zones.push(Zone {
        id: ZoneId::StrId(ids[i].clone()),
        region: Some(region),
        center: Some(Point::new(self.center_x[i], self.center_y[i])),
        vertex_count: Some(self.vertex_count[i]),
        children,
        neighbors,
        area_sqm: Some(0.0), // TODO: New parameter, needs to be reviewed
      })
    }

    Zones { zones: zones }
  }
}

pub struct ZonesWrapper {
  pub inner: Zones,
}

// @TODO needs to be reviewed
impl ZonesWrapper {
  /// Flatten `Zones` into `ZonesExport`:
  /// - Ids are concatenated into utf8_ids w/ id_offsets
  /// - Centers, vertex_count repeated per zone
  /// - Regions flattened with region_offsets
  /// - children/neighbors represented as indices into the zone list
  pub fn to_export(&self) -> JsZones {
    let n = self.inner.zones.len();

    let mut id_offsets = Vec::with_capacity(n);
    let mut utf8_ids = Vec::new();

    let mut center_x = Vec::with_capacity(n);
    let mut center_y = Vec::with_capacity(n);
    let mut vertex_count = Vec::with_capacity(n);

    let mut region_offsets = Vec::with_capacity(n);
    let mut region_coords = Vec::new();

    let mut children_offsets = Vec::new();
    let mut children_id_offsets = Vec::new();
    let mut children_utf8_ids = Vec::new();

    let mut neighbors_offsets = Vec::new();
    let mut neighbors_id_offsets = Vec::new();
    let mut neighbors_utf8_ids = Vec::new();

    let mut area_sqm = Vec::with_capacity(n);

    for zone in &self.inner.zones {
      // --- id ---
      // size of ids
      id_offsets.push(utf8_ids.len() as u32);
      // ids array
      let id_str = zone.id.to_string(); // ZoneID implements Display
      utf8_ids.extend_from_slice(id_str.as_bytes());
      // optionally add a separator if you need readable boundaries, but offsets suffice
      // no separator to save space
      // centers & vertex_count
      // --- center ---

      if let Some(c) = zone.center {
        center_x.push(c.lon);
        center_y.push(c.lat);
      }
      // --- vertex count ---
      if let Some(vc) = zone.vertex_count {
        vertex_count.push(vc);
      }
      // --- region (just exterior ring) ---
      // region exterior ring flattened (x,y)
      region_offsets.push(region_coords.len() as u32);
      // Use exterior ring points (you may want interior rings too depending on your data)

      if let Some(r) = &zone.region {
        for coord in r.exterior.iter() {
          region_coords.push(coord.lon);
          region_coords.push(coord.lat);
        }
      }
      // --- children ---
      // children -> indices
      children_offsets.push(children_id_offsets.len() as u32);
      if let Some(children) = &zone.children {
        for child in children {
          children_id_offsets.push(children_utf8_ids.len() as u32);
          let c = child.to_string();
          children_utf8_ids.extend_from_slice(c.as_bytes());
        }
      }

      // --- neighbors ---
      // neighbors -> indices
      neighbors_offsets.push(neighbors_id_offsets.len() as u32);
      if let Some(neighbors) = &zone.neighbors {
        for neighbor in neighbors {
          neighbors_id_offsets.push(neighbors_utf8_ids.len() as u32);
          let n = neighbor.to_string();
          neighbors_utf8_ids.extend_from_slice(n.as_bytes());
        }
      }
      // --- vertex count ---
      if let Some(area) = zone.area_sqm {
        area_sqm.push(area);
      }
    }

    children_id_offsets.push(children_utf8_ids.len() as u32);
    neighbors_id_offsets.push(neighbors_utf8_ids.len() as u32);

    JsZones {
      id_offsets,
      utf8_ids,
      center_x,
      center_y,
      vertex_count,
      region_offsets,
      region_coords,
      children_offsets,
      children_id_offsets,
      children_utf8_ids,
      neighbors_offsets,
      neighbors_id_offsets,
      neighbors_utf8_ids,
      area_sqm,
    }
  }
}
