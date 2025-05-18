// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::dggrid::common;
use crate::adapters::dggrid::dggrid::DggridAdapter;
use crate::models::common::Zones;
use crate::ports::dggrs::DggrsPort;
use core::f64;
use geo::geometry::Point;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::debug;
pub const CLIP_CELL_DENSIFICATION: u8 = 50; // DGGRID option

pub struct Isea3hImpl {
    pub adapter: DggridAdapter,
}

impl Isea3hImpl {
    // Optional: allow custom paths too
    pub fn new(executable: PathBuf, workdir: PathBuf) -> Self {
        Self {
            adapter: DggridAdapter::new(executable, workdir),
        }
    }
}

impl Default for Isea3hImpl {
    fn default() -> Self {
        Self {
            adapter: DggridAdapter::default(),
        }
    }
}

impl DggrsPort for Isea3hImpl {
    fn zones_from_bbox(
        &self,
        dggs_res_spec: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> Zones {
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, _input_path) =
            common::dggrid_setup(&self.adapter.workdir);

        let _ = common::dggrid_metafile(
            &meta_path,
            &dggs_res_spec,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            densify,
        );

        let _ = isea3h_metafile(&meta_path);

        if let Some(bbox) = &bbox {
            let _ = common::bbox_to_aigen(bbox, &bbox_path);

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

        common::print_file(meta_path.clone());
        common::dggrid_execute(&self.adapter.executable, &meta_path);
        let result =
            common::dggrid_parse(&aigen_path, &children_path, &neighbor_path, &dggs_res_spec);
        common::dggrid_cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
        );
        result
    }

    fn zone_from_point(&self, dggs_res_spec: u8, point: Point, densify: bool) -> Zones {
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            common::dggrid_setup(&self.adapter.workdir);

        let _ = common::dggrid_metafile(
            &meta_path,
            &dggs_res_spec,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            densify,
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
        let _ = writeln!(input_file, "{} {}", point.y(), point.x())
            .expect("Cannot create point input file");

        common::print_file(meta_path.clone());
        common::dggrid_execute(&self.adapter.executable, &meta_path);
        let result =
            common::dggrid_parse(&aigen_path, &children_path, &neighbor_path, &dggs_res_spec);
        common::dggrid_cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
        );
        let _ = fs::remove_file(&input_path);
        result
    }
    fn zones_from_parent(
        &self,
        dggs_res_spec: u8,
        clip_cell_addresses: String, // ToDo: needs validation function
        // clip_cell_res: u8,
        densify: bool,
    ) -> Zones {
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, _input_path) =
            common::dggrid_setup(&self.adapter.workdir);

        let _ = common::dggrid_metafile(
            &meta_path,
            &dggs_res_spec,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            densify,
        );

        let _ = isea3h_metafile(&meta_path);

        // Append to metafile format
        let mut meta_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&meta_path)
            .expect("cannot open file");

        let clip_cell_res = extract_res_from_cellid(&clip_cell_addresses, "ISEA3H").unwrap();

        let clip_cell_address = &clip_cell_addresses[2..]; // strip first two characters. ToDo: can we get the res from the index itself?

        let _ = writeln!(meta_file, "clip_subset_type zones_from_parent");
        let _ = writeln!(meta_file, "clip_cell_res {:?}", clip_cell_res);
        let _ = writeln!(
            meta_file,
            "clip_cell_densification {}",
            CLIP_CELL_DENSIFICATION
        );
        let _ = writeln!(meta_file, "clip_cell_addresses \"{}\"", clip_cell_address);
        let _ = writeln!(meta_file, "input_address_type Z3");
        common::print_file(meta_path.clone());
        common::dggrid_execute(&self.adapter.executable, &meta_path);
        let result =
            common::dggrid_parse(&aigen_path, &children_path, &neighbor_path, &dggs_res_spec);
        common::dggrid_cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
        );
        result
    }
    fn zone_from_id(
        &self,
        zone_id: String, // ToDo: needs validation function
        densify: bool,
    ) -> Zones {
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            common::dggrid_setup(&self.adapter.workdir);

        let clip_cell_res = extract_res_from_cellid(&zone_id, "ISEA3H").unwrap();
        let dggs_res_spec = clip_cell_res;
        let _ = common::dggrid_metafile(
            &meta_path,
            &dggs_res_spec,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            densify,
        );

        let _ = isea3h_metafile(&meta_path);

        // Append to metafile format
        let mut meta_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&meta_path)
            .expect("cannot open file");

        let zone = &zone_id[2..]; // strip first two characters. ToDo: only if we attached the res to the front

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
        let _ = writeln!(input_file, "{}", zone).expect("Cannot create zone id input file");

        let _ = writeln!(meta_file, "dggrid_operation TRANSFORM_POINTS");
        let _ = writeln!(meta_file, "input_address_type Z3");
        common::print_file(meta_path.clone());
        common::dggrid_execute(&self.adapter.executable, &meta_path);
        let result =
            common::dggrid_parse(&aigen_path, &children_path, &neighbor_path, &dggs_res_spec);
        common::dggrid_cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
        );
        result
    }
}

pub fn isea3h_metafile(meta_path: &PathBuf) -> io::Result<()> {
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

pub fn extract_res_from_cellid(id: &str, dggs_type: &str) -> Result<u8, String> {
    match dggs_type {
        "ISEA3H" => extract_res_from_z3(id),
        "IGEO7" => extract_res_from_z3(id), // ToDo: As the extraction of the res based on the Z7
        // index does not yet work, I am using the same method as for Z3.
        _ => Err(format!("Unsupported DGGS type: {}", dggs_type)),
    }
}

/// Extract resolution from ISEA3H ID (Z3)
pub fn extract_res_from_z3(id: &str) -> Result<u8, String> {
    if id.len() < 2 {
        return Err("CellID too short to extract resolution".to_string());
    }

    id[..2]
        .parse::<u8>()
        .map_err(|_| "Invalid resolution prefix in CellID".to_string())
}
/// Extract resolution from IGEO7 ID (Z7)
pub fn extract_res_from_z7(id: &str) -> Result<u8, String> {
    match id.len() {
        1 => Ok(0),
        2 => Ok(1),
        _ => {
            let num = u64::from_str_radix(id, 16).map_err(|_| "Invalid hex CellID".to_string())?;

            let shifted = num << 4;

            let lz = shifted.leading_zeros();

            if lz > 63 {
                return Err("Invalid IGEO7 CellID: No resolution mask found".to_string());
            }

            let res = 2 + lz;

            Ok(res as u8)
        }
    }
}
