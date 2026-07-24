// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

//! Safe, idiomatic wrappers over the libhex9 C ABI (`hex9-sys`), plus the
//! `Zone`/`Zones` builder. All of libhex9's state is read-only after
//! [`super::context::ensure_initialised`], so these helpers are thread-safe.
//!
//! A Hex9 cell is addressed by a 16-byte UUID. The **full** UUID (from
//! `hex9_encode`) is the reversible load-bearer used for traversal; the
//! canonical **bin** at a layer is the GeoPlegma `ZoneId`. libhex9 spells the
//! bin as the keyed label `"<x_list>.<T>"` (e.g. `"43527.4"`); the adapter
//! stores it dot-less (`"435274"`) so the deepest (L30) id stays within the
//! 32-character `ZoneId` limit, re-inserting the `.` only when calling libhex9
//! (see [`compact`] / [`keyed_from_compact`]).

use crate::api::DggrsApiConfig;
use crate::error::hex9::Hex9Error;
use crate::types::{Point, Region, Zone, ZoneId, Zones};
use geo::GeodesicArea;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub type Uuid = [u8; 16];

/// Deepest addressable layer of the linked libhex9 (30 reclaimed / 29 legacy).
pub fn lmax() -> i32 {
    // SAFETY: pure accessor, no arguments.
    unsafe { hex9_sys::hex9_lmax() }
}

/// Encode (lon, lat) degrees to the full, reversible deepest-layer UUID.
pub fn encode(lon: f64, lat: f64) -> Uuid {
    let mut out = [0u8; 16];
    // SAFETY: out is a valid 16-byte buffer.
    unsafe {
        hex9_sys::hex9_encode(lon, lat, out.as_mut_ptr());
    }
    out
}

/// Decode a UUID to its representative (lon, lat) in degrees. Bins resolve to
/// the cell centroid via the (post-L30) exact identity path.
pub fn decode(uuid: &Uuid) -> (f64, f64) {
    let (mut lon, mut lat) = (0.0f64, 0.0f64);
    // SAFETY: uuid is 16 bytes; lon/lat are valid out-pointers.
    unsafe {
        hex9_sys::hex9_decode(uuid.as_ptr(), &mut lon, &mut lat);
    }
    (lon, lat)
}

/// Canonical bin (cell key) of `uuid` at `layer`.
pub fn bin_at(uuid: &Uuid, layer: i32) -> Uuid {
    let mut out = [0u8; 16];
    // SAFETY: both buffers are 16 bytes.
    unsafe {
        hex9_sys::hex9_bin(uuid.as_ptr(), layer, out.as_mut_ptr());
    }
    out
}

/// Recover a full (reversible) UUID for the cell a bin names: decode to its
/// interior centroid, then re-encode. Round-trips exactly (`bin(full, L) ==
/// bin`) on the L30 layout — this is the validated traversal primitive.
pub fn full_from_bin(bin: &Uuid) -> Uuid {
    let (lon, lat) = decode(bin);
    encode(lon, lat)
}

/// Human/`ZoneId` label `"<x_list>.<T>"` for the canonical bin of `uuid` at
/// `layer`. Accepts full or bin UUIDs (full ones are resolved to their bin).
pub fn label_key(uuid: &Uuid, layer: i32) -> Result<String, Hex9Error> {
    let mut buf = [0 as c_char; 48];
    // SAFETY: buf is a valid writable buffer of the stated length.
    let n = unsafe {
        hex9_sys::hex9_label_key(uuid.as_ptr(), layer, buf.as_mut_ptr(), buf.len())
    };
    if n < 0 {
        return Err(Hex9Error::Ffi { call: "hex9_label_key", rc: n as i64 });
    }
    // SAFETY: hex9_label_key NUL-terminates within buf on success.
    let s = unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    Ok(s)
}

/// Parse a canonical label back to its bin UUID and layer.
pub fn parse_label(label: &str) -> Result<(Uuid, i32), Hex9Error> {
    let c = CString::new(label).map_err(|_| Hex9Error::NulInLabel(label.to_string()))?;
    let mut out = [0u8; 16];
    // SAFETY: c is a valid NUL-terminated string; out is 16 bytes.
    let layer = unsafe { hex9_sys::hex9_parse_label(c.as_ptr(), out.as_mut_ptr()) };
    if layer < 0 {
        return Err(Hex9Error::InvalidZoneId(label.to_string()));
    }
    Ok((out, layer))
}

/// Closed ring (lon/lat degrees) of the cell `uuid` at `layer`. `densify >= 0`
/// subdivides each edge into 3^densify segments.
pub fn cell_ring(uuid: &Uuid, layer: i32, densify: i32) -> Result<Vec<Point>, Hex9Error> {
    // SAFETY: pure arithmetic accessor.
    let npoints = unsafe { hex9_sys::hex9_ring_npoints(densify) };
    if npoints <= 0 {
        return Err(Hex9Error::Ffi { call: "hex9_ring_npoints", rc: npoints as i64 });
    }
    let mut buf = vec![0f64; npoints as usize * 2];
    // SAFETY: buf holds 2*npoints doubles, matching max_points = npoints.
    let got = unsafe {
        hex9_sys::hex9_cell_ring(uuid.as_ptr(), layer, densify, buf.as_mut_ptr(), npoints)
    };
    if got < 0 {
        return Err(Hex9Error::Ffi { call: "hex9_cell_ring", rc: got as i64 });
    }
    // Interleaved (lon, lat) pairs -> Point { lat, lon }.
    let pts = (0..got as usize)
        .map(|i| Point::new(buf[2 * i + 1], buf[2 * i]))
        .collect();
    Ok(pts)
}

/// The (up to 6) edge-adjacent neighbour cells of a **full** UUID, as canonical
/// bin UUIDs at `layer`. (libhex9 rejects bin input here — neighbours are keys,
/// not addresses; pass the full UUID.)
pub fn neighbors_of(full: &Uuid, layer: i32) -> Vec<Uuid> {
    let mut buf = [0u8; 6 * 16];
    // SAFETY: buf is 6*16 bytes as the ABI requires.
    let n = unsafe { hex9_sys::hex9_neighbors(full.as_ptr(), layer, buf.as_mut_ptr()) };
    if n < 0 {
        return Vec::new();
    }
    (0..n as usize)
        .map(|i| {
            let mut u = [0u8; 16];
            u.copy_from_slice(&buf[i * 16..i * 16 + 16]);
            u
        })
        .collect()
}

/// libhex9's keyed label `"<x_list>.<T>"` -> GeoPlegma's compact `ZoneId`
/// `"<x_list><T>"`. The single 3-bit key-tail character is always last, so the
/// `.` is pure punctuation; dropping it keeps the deepest (L30) label within the
/// 32-character `ZoneId::new_str` limit (31 body + 1 tail = 32, vs 33 with the
/// dot). Any trailing whitespace is trimmed.
fn compact(keyed: &str) -> String {
    keyed.trim().replace('.', "")
}

/// Inverse of [`compact`]: re-insert the `.` before the final tail character so
/// `hex9_parse_label` sees its keyed form. Idempotent on already-dotted input.
pub fn keyed_from_compact(id: &str) -> String {
    let id = id.trim();
    if id.contains('.') || id.len() < 2 {
        id.to_string()
    } else {
        // Body digits and the tail are all single-byte ASCII, so byte-slicing
        // at len-1 is safe; the tail is exactly one character.
        let split = id.len() - 1;
        format!("{}.{}", &id[..split], &id[split..])
    }
}

/// The compact `ZoneId` (`StrId`) naming the canonical bin of `uuid` at `layer`.
pub fn zone_id_of(uuid: &Uuid, layer: i32) -> Result<ZoneId, Hex9Error> {
    let id = compact(&label_key(uuid, layer)?);
    ZoneId::new_str(&id).map_err(|_| Hex9Error::InvalidZoneId(id))
}

/// Build `Zones` from a set of **full** UUIDs at a single `layer`, honouring the
/// output `config`. Children are always `None`: libhex9 has no direct children
/// primitive yet — descendants come from `zones_from_parent` (grid + ancestry).
pub fn to_zones(fulls: &[Uuid], layer: i32, cfg: DggrsApiConfig) -> Result<Zones, Hex9Error> {
    let mut zones = Vec::with_capacity(fulls.len());

    for full in fulls {
        let bin = bin_at(full, layer);
        let id = zone_id_of(full, layer)?;

        // One ring serves region, area and vertex_count (densify 0 = corners).
        let ring = if cfg.region || cfg.area_sqm || cfg.vertex_count {
            Some(cell_ring(full, layer, 0)?)
        } else {
            None
        };

        let region = if cfg.region || cfg.area_sqm {
            ring.as_ref().map(|p| Region::new(p.clone()))
        } else {
            None
        };

        let area_sqm = if cfg.area_sqm {
            region
                .as_ref()
                .map(|r| r.to_geo_polygon().geodesic_area_unsigned())
        } else {
            None
        };

        let vertex_count = if cfg.vertex_count {
            // The ring is closed (last == first); unique vertices = len - 1.
            ring.as_ref().map(|p| p.len().saturating_sub(1) as u32)
        } else {
            None
        };

        let center = if cfg.center {
            let (lon, lat) = decode(&bin);
            Some(Point::new(lat, lon))
        } else {
            None
        };

        let neighbors = if cfg.neighbors {
            Some(
                neighbors_of(full, layer)
                    .iter()
                    .map(|nb| zone_id_of(nb, layer))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        } else {
            None
        };

        zones.push(Zone {
            id,
            region,
            center,
            vertex_count,
            children: None,
            neighbors,
            area_sqm,
        });
    }

    Ok(Zones { zones })
}
