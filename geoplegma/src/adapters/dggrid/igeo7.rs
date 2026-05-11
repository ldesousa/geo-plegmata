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

pub struct Igeo7Impl {
    id: DggrsUid,
    adapter: DggridAdapter,
}

impl Igeo7Impl {
    // Optional: allow custom paths too
    pub fn new(executable: PathBuf, workdir: PathBuf) -> Self {
        Self {
            id: DggrsUid::IGEO7,
            adapter: DggridAdapter::new(executable, workdir),
        }
    }
}

impl Default for Igeo7Impl {
    fn default() -> Self {
        Self {
            id: DggrsUid::IGEO7,
            adapter: DggridAdapter::default(),
        }
    }
}

impl DggrsApi for Igeo7Impl {
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

        let _ = igeo7_metafile(&meta_path);

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

        let _ = igeo7_metafile(&meta_path);

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

        let parent_zone_res = get_refinement_level_from_z7_zone_id(&parent_zone_id)?;
        let target_level = parent_zone_res.add(relative_depth)?;

        let _ = common::write::metafile(
            &meta_path,
            &target_level,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            &cfg,
        );

        let _ = igeo7_metafile(&meta_path);

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
        let _ = writeln!(meta_file, "input_address_type Z7");
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

        let child_level = get_refinement_level_from_z7_zone_id(&zone_id)?;
        if child_level.get() == 0 {
            return Err(DggrsError::Dggrid(DggridError::InvalidZ7Format(
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

        let _ = igeo7_metafile(&meta_path);

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
        let _ = writeln!(meta_file, "input_address_type Z7");
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

        let refinement_level = get_refinement_level_from_z7_zone_id(&zone_id).unwrap();
        let _ = common::write::metafile(
            &meta_path,
            &refinement_level,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            &cfg,
        );

        let _ = igeo7_metafile(&meta_path);

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
        let _ = writeln!(meta_file, "input_address_type Z7");
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

pub fn igeo7_metafile(meta_path: &Path) -> io::Result<()> {
    debug!("Writing to {:?}", meta_path);
    // Append to metafile format
    let mut meta_file = OpenOptions::new()
        .append(true)
        .write(true)
        .open(&meta_path)
        .expect("cannot open file");
    writeln!(meta_file, "dggs_type {}", "IGEO7")?;
    writeln!(meta_file, "dggs_aperture 7")?;
    writeln!(meta_file, "output_address_type Z7")?;

    Ok(())
}

/// Determines the refinement level from an IGEO7 (Z7) zone identifier.
///
/// This function interprets a Z7 zone identifier as defined by the Z7 indexing scheme, where the first four bits encode the base cell number and the remaining 60 bits are composed of 20 three-bit digits. Digits with values `0` through `6` represent valid resolution steps, while the value `7` indicates padding beyond the zone’s resolution. The refinement level is determined by counting the number of valid digits before the first padding digit. If no padding digit is found, the maximum refinement level of 20 is returned. See [IGEO7: A new hierarchically indexed hexagonal equal-area discrete global grid system ](https://doi.org/10.5194/agile-giss-6-32-2025) for more information.
///
/// # Parameters
/// - `dggrid_z7_id`: A `ZoneId` expected to be in hexadecimal form (`ZoneId::HexId`).
///
/// # Returns
/// - `Ok(RefinementLevel)`: The detected refinement level.
/// - `Err(DggrsError)`: If the identifier is not a `HexId`, contains invalid digits, or fails to create a valid `RefinementLevel`.
///
/// # Panics
/// This function will panic if the provided hex string cannot be parsed into a `u64`, though this is not expected when IDs are generated by DGGRID.
///
/// # Requirements
/// Zone identifiers must be generated using DGGRID version 8.41 or later to ensure compatibility with the Z7 format.
pub fn get_refinement_level_from_z7_zone_id(
    dggrid_z7_id: &ZoneId,
) -> Result<RefinementLevel, DggrsError> {
    // make sure to generate zones with DGGRID version 8.41
    let hex = match dggrid_z7_id {
        ZoneId::HexId(h) => h.as_str(),
        _ => {
            return Err(DggrsError::Dggrid(DggridError::InvalidZ7Format(
                "Expected ZoneId::HexId".to_string(),
            )))?;
        }
    };

    let v = u64::from_str_radix(hex, 16).unwrap(); // NOTE: This should be safe if the hex string is coming from DGGRID.

    let mut resolution = RefinementLevel::new(20)?;
    for i in 0..20 {
        let shift = 60 - 3 * (i + 1);
        let digit = ((v >> shift) & 0b111) as u8;

        if digit > 7 {
            return Err(DggrsError::Dggrid(DggridError::InvalidZ7Format(format!(
                "Invalid Z7 digit {} at position {}",
                digit,
                i + 1
            ))));
        }
        if digit == 7 {
            resolution = RefinementLevel::new(i)?;
            break;
        }
    }

    Ok(resolution)
}
