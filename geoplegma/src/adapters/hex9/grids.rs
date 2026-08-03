// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::hex9::common::{
    bin_at, cell_ring, encode, full_from_bin, keyed_from_compact, parse_label, to_zones, Uuid,
};
use crate::adapters::hex9::context::ensure_initialised;
use crate::api::{DggrsApi, DggrsApiConfig};
use crate::error::DggrsError;
use crate::error::hex9::Hex9Error;
use crate::types::{
    BoundingBox, DggrsName, DggrsUid, Point, RefinementLevel, RelativeDepth, ZoneId, Zones,
};
use std::ffi::c_char;

/// Cap on a single grid enumeration (guards a runaway whole-world deep grid).
/// libhex9 bails mid-BFS once this is exceeded, so it fails fast rather than
/// trying to materialise 12·9^layer cells. Hefty but bounded: ~100M cells is a
/// few GB of geometry — a guard-rail, not a target (real queries are far smaller).
const GRID_MAX_CELLS: i64 = 100_000_000;

/// The libhex9-backed (Hex9 / H9 aperture-9 hexagonal DGGS) adapter.
pub struct Hex9Impl {
    pub id: DggrsUid,
}

impl Hex9Impl {
    pub fn new(id: DggrsUid) -> Self {
        ensure_initialised();
        Self { id }
    }

    #[inline]
    fn grid_name(&self) -> DggrsName {
        self.id.spec().name
    }

    fn check_level(&self, level: RefinementLevel) -> Result<(), DggrsError> {
        if level > self.max_refinement_level()? {
            return Err(DggrsError::RefinementLevelLimitReached {
                grid_name: self.grid_name().to_string(),
                requested: level,
                maximum: self.max_refinement_level()?,
            });
        }
        Ok(())
    }

    /// Hex9-specific (NOT part of the `DggrsApi` contract): the boundary ring of
    /// a single zone, densified by `densify` (0..=9 → each edge split into
    /// 3^densify segments, i.e. `6 * 3^densify + 1` points). The generic
    /// `DggrsApiConfig` only carries a densify on/off bool and can't express the
    /// amount, so a caller holding a concrete `Hex9Impl` uses this to choose it.
    pub fn zone_ring(&self, zone_id: ZoneId, densify: i32) -> Result<Vec<Point>, DggrsError> {
        let (bin, layer) = resolve(&zone_id)?;
        let full = full_from_bin(&bin);
        Ok(cell_ring(&full, layer, densify)?)
    }
}

/// RAII handle over a libhex9 grid enumeration.
struct Grid {
    ptr: *mut hex9_sys::hex9_grid,
}

impl Grid {
    fn create(bbox: &BoundingBox, layer: i32) -> Result<Self, Hex9Error> {
        let mut err = [0 as c_char; 256];
        // SAFETY: all scalars by value; err is a valid writable buffer.
        let ptr = unsafe {
            hex9_sys::hex9_grid_create(
                bbox.min_lon,
                bbox.min_lat,
                bbox.max_lon,
                bbox.max_lat,
                layer,
                0, // densify default; rings are built on demand
                GRID_MAX_CELLS,
                err.as_mut_ptr(),
                err.len(),
            )
        };
        if ptr.is_null() {
            // SAFETY: err is NUL-terminated by the ABI on failure.
            let msg = unsafe { std::ffi::CStr::from_ptr(err.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            return Err(Hex9Error::Grid(msg));
        }
        Ok(Self { ptr })
    }

    fn count(&self) -> i32 {
        // SAFETY: ptr is a live handle.
        unsafe { hex9_sys::hex9_grid_count(self.ptr as *const _) }
    }

    /// Full, reversible identity UUID of grid cell `i`.
    fn cell_id(&self, i: i32) -> Uuid {
        let mut out = [0u8; 16];
        // SAFETY: i is in 0..count; out is 16 bytes.
        unsafe { hex9_sys::hex9_grid_cell_id(self.ptr as *const _, i, out.as_mut_ptr()) };
        out
    }
}

impl Drop for Grid {
    fn drop(&mut self) {
        // SAFETY: ptr was returned by hex9_grid_create and not yet freed.
        unsafe { hex9_sys::hex9_grid_destroy(self.ptr) };
    }
}

/// Min/max lon/lat of a ring, padded outward so the centroid-in-bbox grid
/// filter cannot miss the cell's own descendants on the boundary.
fn bbox_of(ring: &[Point]) -> BoundingBox {
    let mut min_lon = f64::INFINITY;
    let mut min_lat = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    for p in ring {
        min_lon = min_lon.min(p.lon);
        max_lon = max_lon.max(p.lon);
        min_lat = min_lat.min(p.lat);
        max_lat = max_lat.max(p.lat);
    }
    let pad = 1e-9_f64.max((max_lon - min_lon) * 1e-6);
    BoundingBox::new(
        (min_lon - pad).max(-180.0),
        (min_lat - pad).max(-90.0),
        (max_lon + pad).min(180.0),
        (max_lat + pad).min(90.0),
    )
}

/// Resolve a `ZoneId` to its canonical bin UUID and layer. Hex9 ids are textual
/// labels (`"<x_list>.<T>"`); integer ids are not a Hex9 address.
fn resolve(zone_id: &ZoneId) -> Result<(Uuid, i32), Hex9Error> {
    match zone_id {
        ZoneId::StrId(s) => parse_label(&keyed_from_compact(s)),
        ZoneId::HexId(h) => parse_label(&keyed_from_compact(h.as_str())),
        ZoneId::IntId(i) => Err(Hex9Error::UnsupportedZoneId(*i)),
    }
}

impl DggrsApi for Hex9Impl {
    fn zones_from_bbox(
        &self,
        refinement_level: RefinementLevel,
        bbox: Option<BoundingBox>,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        self.check_level(refinement_level)?;
        let layer = refinement_level.get();
        let extent = bbox.unwrap_or(BoundingBox::WORLD);

        let grid = Grid::create(&extent, layer)?;
        let count = grid.count();
        let fulls: Vec<Uuid> = (0..count).map(|i| grid.cell_id(i)).collect();
        Ok(to_zones(&fulls, layer, cfg)?)
    }

    fn zone_from_point(
        &self,
        refinement_level: RefinementLevel,
        point: Point,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        self.check_level(refinement_level)?;
        let full = encode(point.lon, point.lat);
        Ok(to_zones(&[full], refinement_level.get(), cfg)?)
    }

    fn zones_from_parent(
        &self,
        relative_depth: RelativeDepth,
        parent_zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();

        if relative_depth > self.max_relative_depth()? {
            return Err(DggrsError::RelativeDepthLimitReached {
                grid_name: self.grid_name().to_string(),
                requested: relative_depth,
                maximum: self.max_relative_depth()?,
            });
        }

        let (parent_bin, parent_layer) = resolve(&parent_zone_id)?;
        let target = RefinementLevel::new(parent_layer)?.add(relative_depth)?;
        if target > self.max_refinement_level()? {
            return Err(DggrsError::RefinementLevelPlusRelativeDepthLimitReached {
                grid_name: self.grid_name().to_string(),
                requested: relative_depth,
                maximum: self.max_refinement_level()?,
            });
        }

        // Enumerate the target layer over the parent's bounding box, then keep
        // only cells whose ancestor at the parent layer IS the parent — exact
        // containment-based descent (the decode->re-encode->bin traversal rule).
        let parent_full = full_from_bin(&parent_bin);
        let ring = cell_ring(&parent_full, parent_layer, 0)?;
        let grid = Grid::create(&bbox_of(&ring), target.get())?;

        let mut descendants: Vec<Uuid> = Vec::new();
        for i in 0..grid.count() {
            let cand = grid.cell_id(i);
            if bin_at(&cand, parent_layer) == parent_bin {
                descendants.push(cand);
            }
        }
        Ok(to_zones(&descendants, target.get(), cfg)?)
    }

    fn primary_parent_from_zone(
        &self,
        zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let (bin, layer) = resolve(&zone_id)?;
        if layer < 1 {
            return Err(Hex9Error::NoParent.into());
        }

        // Primary parent = the mode-0 cell that geometrically contains this one,
        // via decode -> re-encode -> bin(L-1). `primary_parent_from_zone` is
        // documented as implementation-defined; containment is the Hex9 choice.
        let full = full_from_bin(&bin);
        let parent_layer = layer - 1;
        let parent_bin = bin_at(&full, parent_layer);
        let parent_full = full_from_bin(&parent_bin);
        Ok(to_zones(&[parent_full], parent_layer, cfg)?)
    }

    fn zone_from_id(
        &self,
        zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let (bin, layer) = resolve(&zone_id)?;
        let full = full_from_bin(&bin);
        Ok(to_zones(&[full], layer, cfg)?)
    }

    fn zone_count(&self, refinement_level: RefinementLevel) -> Result<u64, DggrsError> {
        // 12 base cells, aperture 9: 12 * 9^level (saturates rather than panics
        // at the deepest layers, where the exact count overflows u64).
        let level = refinement_level.get().max(0) as u32;
        Ok(12u64.saturating_mul(9u64.saturating_pow(level)))
    }

    fn min_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        Ok(self.id.spec().min_refinement_level)
    }

    fn max_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        Ok(self.id.spec().max_refinement_level)
    }

    fn default_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        Ok(self.id.spec().default_refinement_level)
    }

    fn max_relative_depth(&self) -> Result<RelativeDepth, DggrsError> {
        Ok(self.id.spec().max_relative_depth)
    }

    fn default_relative_depth(&self) -> Result<RelativeDepth, DggrsError> {
        Ok(self.id.spec().default_relative_depth)
    }
}
