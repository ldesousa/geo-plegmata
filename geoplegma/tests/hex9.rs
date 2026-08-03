// Copyright 2025 contributors to the GeoPlegma project.
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your discretion.

//! API tests for the Hex9 (libhex9 / H9) adapter.
//!
//! Hex9 differs from the other backends:
//!   * Refinement levels run 0..=30 — the 12 base hexagons are the well-defined
//!     L0. The *addressing* API (`zone_from_point`, `zone_from_id`,
//!     `primary_parent_from_zone`) works across the whole range.
//!   * The *enumeration* API (`zones_from_bbox`, `zones_from_parent`) goes
//!     through libhex9's `hex9_grid_create`, which the currently linked build
//!     restricts to layers 1..29 — so L0 / deep layers cannot be enumerated.
//!   * `to_zones` never populates `children` (libhex9 has no direct children
//!     enumeration), so the dggal-style children helpers do not apply.

use geoplegma::adapters::hex9::grids::Hex9Impl;
use geoplegma::api::{DggrsApi, DggrsApiConfig};
use geoplegma::types::{DggrsUid, Point, RefinementLevel, ZoneId};

const PLAIN: DggrsApiConfig = DggrsApiConfig {
    area_sqm: false,
    densify: false,
    center: false,
    region: false,
    children: false,
    neighbors: false,
    vertex_count: false,
};

const POINT: Point = Point { lat: 52.98, lon: 9.06 };

/// Closed-form zone count is 12 · 9^level at every layer, including L0 = the 12
/// base hexagons.
#[test]
fn hex9_zone_count_closed_form() {
    let hex9 = Hex9Impl::new(DggrsUid::HEX9);

    for level_int in 0..=15 {
        let level = RefinementLevel::new(level_int).unwrap();
        let expected = 12u64 * 9u64.pow(level_int as u32);

        assert_eq!(
            hex9.zone_count(level).unwrap(),
            expected,
            "L{level_int}: zone_count must be 12 * 9^{level_int}",
        );
    }
}

/// The closed form must equal the whole-world cell count, including L0 (the 12
/// base hexagons). Exercises `zones_from_bbox`.
#[test]
fn hex9_zone_count_matches_world_enumeration() {
    let hex9 = Hex9Impl::new(DggrsUid::HEX9);

    for level_int in 0..=3 {
        let level = RefinementLevel::new(level_int).unwrap();

        let expected = hex9.zone_count(level).unwrap();
        // Default config (renders cell rings) — also smoke-tests L0 geometry.
        let zones = hex9.zones_from_bbox(level, None, None).unwrap();

        assert_eq!(
            expected,
            zones.zones.len() as u64,
            "L{level_int}: closed-form zone_count must equal the whole-world grid size",
        );
    }
}

/// A zone addressed by a point must survive a textual round-trip: re-resolving
/// its own id via `zone_from_id` yields the same id. Exercises `zone_from_point`
/// + `zone_from_id` and the label <-> bin encoding across the full 0..=30
/// addressing range (these use encode()/labels, not grid_create).
#[test]
fn hex9_zone_from_id_round_trips() {
    let hex9 = Hex9Impl::new(DggrsUid::HEX9);
    let max = hex9.max_refinement_level().unwrap().get();

    for rf in 0..=max {
        let level = RefinementLevel::new(rf).unwrap();
        let id = point_zone(&hex9, level.get(), POINT);

        let round_tripped = hex9
            .zone_from_id(id.clone(), Some(PLAIN))
            .unwrap()
            .zones
            .first()
            .unwrap()
            .id
            .clone();

        assert_eq!(id, round_tripped, "L{rf}: zone_from_id must round-trip the zone id");
    }
}

/// `primary_parent_from_zone` steps down exactly one level per call: iterated
/// from a deep cell it must reach an L0 base hexagon in exactly `deep` steps,
/// after which an L0 cell has no parent. Exercises `zone_from_point` +
/// `primary_parent_from_zone` over its whole range.
#[test]
fn hex9_primary_parent_climbs_to_base() {
    let hex9 = Hex9Impl::new(DggrsUid::HEX9);
    let deep = 20;

    let mut zone = point_zone(&hex9, deep, POINT);
    for _ in 0..deep {
        zone = primary_parent(&hex9, zone);
    }

    // `zone` is now the L0 ancestor; an L0 base hexagon has no primary parent.
    assert!(
        hex9.primary_parent_from_zone(zone, Some(PLAIN)).is_err(),
        "an L0 base hexagon must have no primary parent",
    );
}

/// An L0 base hexagon's neighbours are queryable (the libhex9 kring layer gate
/// now admits L0). Exercises `zone_from_point` with `neighbors` at L0.
#[test]
fn hex9_l0_base_hexagon_has_neighbours() {
    let hex9 = Hex9Impl::new(DggrsUid::HEX9);
    let cfg = DggrsApiConfig { neighbors: true, ..PLAIN };

    let zone = hex9
        .zone_from_point(RefinementLevel::new(0).unwrap(), POINT, Some(cfg))
        .unwrap()
        .zones
        .into_iter()
        .next()
        .unwrap();

    let neighbours = zone.neighbors.expect("neighbours were requested");
    assert!(
        !neighbours.is_empty(),
        "an L0 base hexagon must have queryable neighbours",
    );
}

/// A deep whole-world enumeration must bail against the cell ceiling instead of
/// trying to materialise 12·9^level cells. Tested at the libhex9 FFI boundary
/// with a tiny budget so the bail is instant (the adapter's real ceiling is
/// 100M, which would be valid but multi-GB to drive — the mechanism is the same).
#[test]
fn hex9_grid_create_bails_against_max_cells() {
    let mut err = [0 as std::os::raw::c_char; 256];
    // L20 over the whole globe is ~1.5e19 cells; a 10k budget must bail at once.
    let grid = unsafe {
        hex9_sys::hex9_grid_create(
            -180.0, -90.0, 180.0, 90.0, // WORLD
            /*layer=*/ 20,
            /*densify=*/ 0,
            /*max_cells=*/ 10_000,
            err.as_mut_ptr(),
            err.len(),
        )
    };
    assert!(
        grid.is_null(),
        "a deep whole-world enumeration must bail against the max_cells budget",
    );
}

/// Hex9-specific densify control via an inherent method on the concrete type
/// (outside the `DggrsApi` contract). `densify` splits each edge into 3^densify
/// segments, so the ring has `6 * 3^densify + 1` points.
#[test]
fn hex9_zone_ring_densify_selects_resolution() {
    let hex9 = Hex9Impl::new(DggrsUid::HEX9);
    let zone = point_zone(&hex9, 6, POINT);

    let corners = hex9.zone_ring(zone.clone(), 0).unwrap();
    let dense = hex9.zone_ring(zone, 2).unwrap();

    assert_eq!(corners.len(), 7, "densify 0 = 6 hexagon corners + closing point");
    assert_eq!(dense.len(), 55, "densify 2 = 6 * 3^2 + 1 points");
}

fn point_zone(hex9: &Hex9Impl, level: i32, point: Point) -> ZoneId {
    hex9.zone_from_point(RefinementLevel::new(level).unwrap(), point, Some(PLAIN))
        .unwrap()
        .zones
        .first()
        .unwrap()
        .id
        .clone()
}

fn primary_parent(hex9: &Hex9Impl, zone: ZoneId) -> ZoneId {
    hex9.primary_parent_from_zone(zone, Some(PLAIN))
        .unwrap()
        .zones
        .first()
        .unwrap()
        .id
        .clone()
}
