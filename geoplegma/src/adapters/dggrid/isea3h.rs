// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::dggrid::common;
use crate::adapters::dggrid::dggrid::DggridAdapter;
use crate::api::{DggrsApi, DggrsApiConfig};
use crate::error::DggrsError;
use crate::error::dggrid::DggridError;
use crate::types::{BoundingBox, DggrsUid, Point, RefinementLevel, RelativeDepth, ZoneId, Zones};
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::debug;
pub const CLIP_CELL_DENSIFICATION: u8 = 50; // DGGRID option

pub struct Isea3hImpl {
    id: DggrsUid,
    adapter: DggridAdapter,
}

impl Isea3hImpl {
    // Optional: allow custom paths too
    pub fn new(executable: PathBuf, workdir: PathBuf) -> Self {
        Self {
            id: DggrsUid::ISEA3HDGGRID,
            adapter: DggridAdapter::new(executable, workdir),
        }
    }
}

impl Default for Isea3hImpl {
    fn default() -> Self {
        Self {
            id: DggrsUid::ISEA3HDGGRID,
            adapter: DggridAdapter::default(),
        }
    }
}

impl DggrsApi for Isea3hImpl {
    fn zones_from_bbox(
        &self,
        refinement_level: RefinementLevel,
        bbox: Option<BoundingBox>,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            common::dggrid::setup(&self.adapter.workdir);

        let _ = common::write::metafile(
            &meta_path,
            &refinement_level,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            &cfg,
        );

        let _ = isea3h_metafile(&meta_path);

        if let Some(bbox) = &bbox {
            let _ = common::write::bbox(bbox, &bbox_path);

            // Append to metafile
            let mut meta_file = OpenOptions::new()
                .append(true)
                .write(true)
                .open(&meta_path)
                .expect("cannot open file");

            let _ = writeln!(meta_file, "clip_subset_type AIGEN");
            let _ = writeln!(
                meta_file,
                "clip_region_files {}",
                &bbox_path.to_string_lossy()
            );
        }

        common::write::file(&meta_path);
        common::dggrid::execute(&self.adapter.executable, &meta_path);
        let result = common::output::ingest(&aigen_path, &children_path, &neighbor_path, &cfg)?;
        common::cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
            &input_path,
        );
        Ok(result)
    }

    fn zone_from_point(
        &self,
        refinement_level: RefinementLevel,
        point: Point,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            common::dggrid::setup(&self.adapter.workdir);

        let _ = common::write::metafile(
            &meta_path,
            &refinement_level,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            &cfg,
        );

        let _ = isea3h_metafile(&meta_path);

        // Append to metafile
        let mut meta_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&meta_path)
            .expect("cannot open file");

        let _ = writeln!(meta_file, "dggrid_operation TRANSFORM_POINTS");
        let _ = writeln!(meta_file, "input_address_type GEO");
        let _ = writeln!(
            meta_file,
            "input_file_name {}",
            &input_path.to_string_lossy()
        );

        // File with one point
        let mut input_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&input_path)
            .expect("cannot open file");
        let _ = writeln!(input_file, "{} {}", point.lon, point.lat)
            .expect("Cannot create point input file");

        common::write::file(&meta_path);
        common::dggrid::execute(&self.adapter.executable, &meta_path);
        let result = common::output::ingest(&aigen_path, &children_path, &neighbor_path, &cfg)?;
        common::cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
            &input_path,
        );
        let _ = fs::remove_file(&input_path);
        Ok(result)
    }
    fn zones_from_parent(
        &self,
        relative_depth: RelativeDepth,
        parent_zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            common::dggrid::setup(&self.adapter.workdir);

        let parent_zone_res = get_refinement_level_from_z3_zone_id(&parent_zone_id)?;
        let target_level = parent_zone_res.add(relative_depth)?;

        let _ = common::write::metafile(
            &meta_path,
            &target_level,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            &cfg,
        );

        let _ = isea3h_metafile(&meta_path);

        // Append to metafile format
        let mut meta_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&meta_path)
            .expect("cannot open file");

        let _ = writeln!(meta_file, "clip_subset_type COARSE_CELLS");
        let _ = writeln!(meta_file, "clip_cell_res {:?}", parent_zone_res.get());
        let _ = writeln!(
            meta_file,
            "clip_cell_densification {}",
            CLIP_CELL_DENSIFICATION
        );
        let _ = writeln!(meta_file, "clip_cell_addresses \"{}\"", parent_zone_id);
        let _ = writeln!(meta_file, "input_address_type Z3");
        common::write::file(&meta_path);
        common::dggrid::execute(&self.adapter.executable, &meta_path);
        let result = common::output::ingest(&aigen_path, &children_path, &neighbor_path, &cfg)?;
        common::cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
            &input_path,
        );
        Ok(result)
    }

    fn primary_parent_from_zone(
        &self,
        zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            common::dggrid::setup(&self.adapter.workdir);

        let child_level = get_refinement_level_from_z3_zone_id(&zone_id)?;
        if child_level.get() == 0 {
            return Err(DggrsError::Dggrid(DggridError::InvalidZ3Format(
                "Root-level zones do not have a parent".to_string(),
            )));
        }
        let parent_level = RefinementLevel::new(child_level.get() - 1)?;

        let _ = common::write::metafile(
            &meta_path,
            &parent_level,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            &cfg,
        );

        let _ = isea3h_metafile(&meta_path);

        let mut meta_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&meta_path)
            .expect("cannot open file");

        let _ = writeln!(
            meta_file,
            "input_file_name {}",
            &input_path.to_string_lossy()
        );

        let mut input_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&input_path)
            .expect("cannot open file");
        let _ = writeln!(input_file, "{}", zone_id).expect("Cannot create zone id input file");

        let _ = writeln!(meta_file, "dggrid_operation TRANSFORM_POINTS");
        let _ = writeln!(meta_file, "input_address_type Z3");
        common::write::file(&meta_path);
        common::dggrid::execute(&self.adapter.executable, &meta_path);
        let result = common::output::ingest(&aigen_path, &children_path, &neighbor_path, &cfg)?;
        common::cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
            &input_path,
        );
        Ok(result)
    }

    fn zone_from_id(
        &self,
        zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            common::dggrid::setup(&self.adapter.workdir);

        let refinement_level = get_refinement_level_from_z3_zone_id(&zone_id)?;
        let _ = common::write::metafile(
            &meta_path,
            &refinement_level,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            &cfg,
        );

        let _ = isea3h_metafile(&meta_path);

        // Append to metafile format
        let mut meta_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&meta_path)
            .expect("cannot open file");

        let _ = writeln!(
            meta_file,
            "input_file_name {}",
            &input_path.to_string_lossy()
        );

        // File with one point
        let mut input_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&input_path)
            .expect("cannot open file");
        let _ = writeln!(input_file, "{}", zone_id).expect("Cannot create zone id input file");

        let _ = writeln!(meta_file, "dggrid_operation TRANSFORM_POINTS");
        let _ = writeln!(meta_file, "input_address_type Z3");
        common::write::file(&meta_path);
        common::dggrid::execute(&self.adapter.executable, &meta_path);
        let result = common::output::ingest(&aigen_path, &children_path, &neighbor_path, &cfg)?;
        common::cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
            &input_path,
        );
        Ok(result)
    }

    fn zone_count(&self, refinement_level: RefinementLevel) -> Result<u64, DggrsError> {
        let r = refinement_level.get();
        let aperture: u64 = self.id.spec().aperture.into();
        Ok(2 + 10 * (aperture.pow(r as u32)))
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

pub fn isea3h_metafile(meta_path: &Path) -> io::Result<()> {
    debug!("Writing to {:?}", meta_path);
    // Append to metafile format
    let mut meta_file = OpenOptions::new()
        .append(true)
        .write(true)
        .open(&meta_path)
        .expect("cannot open file");
    writeln!(meta_file, "dggs_type {}", "ISEA3H")?;
    writeln!(meta_file, "dggs_aperture 3")?;
    writeln!(meta_file, "output_address_type Z3")?;

    Ok(())
}

/// Determines the refinement level from an ISEA3H (Z3) zone identifier.
///
/// This function decodes a Z3 zone identifier generated by DGGRID with `z3_invalid_digit` set to `3`. In the Z3 format, the first four bits of the 64-bit integer encode the base cell number, and the remaining 60 bits are split into 30 two-bit digits. Digits with values `0` through `2` represent valid resolution steps, while the value `3` indicates padding beyond the zone’s resolution.
///
/// The refinement level is calculated by counting the number of valid digits before the first padding digit. If no padding digit is found, the maximum refinement level of 30 is returned.
///
/// # Parameters
/// - `dggrid_z3_id`: A `ZoneId` expected to be in hexadecimal form (`ZoneId::HexId`).
///
/// # Returns
/// - `Ok(RefinementLevel)`: The detected refinement level.
/// - `Err(GeoPlegmaError)`: If the identifier is not a `HexId`, contains invalid digits, or fails to create a valid `RefinementLevel`.
///
/// # Panics
/// This function will panic if the provided hex string cannot be parsed into a `u64`, though this is not expected when IDs are generated by DGGRID with the proper configuration.
///
/// # Requirements
/// Zone identifiers must be generated using DGGRID version 8.41 or later, with `z3_invalid_digit` explicitly set to `3`.
pub fn get_refinement_level_from_z3_zone_id(
    dggrid_z3_id: &ZoneId,
) -> Result<RefinementLevel, DggrsError> {
    // make sure to generate zones with DGGRID version 8.41 and z3_invalid_digit 3
    let hex = match dggrid_z3_id {
        ZoneId::HexId(h) => h.as_str(),
        _ => {
            return Err(DggrsError::Dggrid(DggridError::InvalidZ3Format(
                "Expected ZoneId::HexId".to_string(),
            )))?;
        }
    };

    let v = u64::from_str_radix(hex, 16).unwrap(); // NOTE: This should be safe if the hex string is coming from DGGRID.
    let mut resolution = RefinementLevel::new(30)?; // default if no padding found
    for i in 0..30 {
        let shift = 60 - 2 * (i + 1);
        let digit = ((v >> shift) & 0b11) as u64;

        if digit > 3 {
            return Err(DggrsError::Dggrid(DggridError::InvalidZ3Format(format!(
                "Invalid Z3 digit {} at position {}",
                digit,
                i + 1
            ))));
        }
        if digit == 3 {
            resolution = RefinementLevel::new(i)?;
            break;
        }
    }

    Ok(resolution)
}
