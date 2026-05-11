// Copyright 2025 iontributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod dggrid {
    use rand::distributions::{Alphanumeric, DistString};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    pub fn setup(workdir: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
        let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        let meta_path = workdir.join(&code).with_extension("meta"); // metafile
        let aigen_path = workdir.join(&code).with_extension("gen"); // AIGEN
        let children_path = workdir.join(&code).with_extension("chd"); // Children
        let neighbor_path = workdir.join(&code).with_extension("nbr"); // Neightbors
        let bbox_path = workdir.join(&code).with_extension("bbox"); // BBox
        let input_path = workdir.join(&code).with_extension("txt"); // Input file for e.g. points
        (
            meta_path,
            aigen_path,
            children_path,
            neighbor_path,
            bbox_path,
            input_path,
        )
    }
    pub fn execute(dggrid_path: &Path, meta_path: &Path) {
        let _ = Command::new(&dggrid_path).arg(&meta_path).output(); // FIX: Better handling of output and raise DggridError::DggridExecutionFailed
    }
}

pub mod write {
    use crate::api::DggrsApiConfig;
    use crate::types::{BoundingBox, RefinementLevel};
    use std::fs;
    use std::io::{self, Write};
    use std::path::Path;
    use tracing::debug;

    pub const DENSIFICATION: u8 = 50; // DGGRID option

    pub fn metafile(
        metafile: &Path,
        refinement_level: &RefinementLevel,
        cell_output_file_name: &Path,
        children_output_file_name: &Path,
        neighbor_output_file_name: &Path,
        conf: &DggrsApiConfig,
    ) -> io::Result<()> {
        debug!("Writing to {:?}", metafile);
        let mut file = fs::File::create(metafile)?;
        writeln!(file, "longitude_wrap_mode UNWRAP_EAST")?;
        writeln!(file, "cell_output_type AIGEN")?;
        writeln!(file, "unwrap_points FALSE")?;
        writeln!(file, "output_cell_label_type OUTPUT_ADDRESS_TYPE")?;
        writeln!(file, "precision 7")?;
        writeln!(file, "dggs_res_spec {}", refinement_level.get())?;
        writeln!(file, "z3_invalid_digit 3")?; // TODO: Remove with DGGRID version 9 as this is  only relevant for ISEA3H and only placed here for convenience.
        writeln!(file, "output_file_type NONE")?;

        writeln!(
            file,
            "cell_output_file_name {}",
            cell_output_file_name.to_string_lossy().into_owned()
        )?;

        if conf.neighbors {
            writeln!(file, "neighbor_output_type TEXT")?;
            writeln!(
                file,
                "neighbor_output_file_name {}",
                neighbor_output_file_name.to_string_lossy().into_owned()
            )?;
        }

        if conf.children {
            writeln!(file, "children_output_type TEXT")?;
            writeln!(
                file,
                "children_output_file_name {}",
                children_output_file_name.to_string_lossy().into_owned()
            )?;
        }

        if conf.densify {
            writeln!(file, "densification {}", DENSIFICATION)?;
        }

        Ok(())
    }

    pub fn bbox(bbox: &BoundingBox, bboxfile: &Path) -> io::Result<()> {
        let (minx, miny) = (bbox.min_lon, bbox.min_lat);
        let (maxx, maxy) = (bbox.max_lon, bbox.max_lat);

        // define the 5 vertices (closing the polygon)
        let vertices = vec![
            (minx, miny), // lower-left
            (maxx, miny), // lower-right
            (maxx, maxy), // upper-right
            (minx, maxy), // upper-left
            (minx, miny), // close
        ];
        let mut file = fs::File::create(bboxfile)?;

        // First line: ID and center of the bbox (NOT part of the ring)
        let center_x = (minx + maxx) / 2.0;
        let center_y = (miny + maxy) / 2.0;
        writeln!(file, "1 {:.7} {:.7}", center_x, center_y)?;

        for (x, y) in &vertices {
            writeln!(file, "{:.7} {:.7}", x, y)?;
        }

        writeln!(file, "END")?;
        writeln!(file, "END")?;

        Ok(())
    }

    pub fn file(file: &Path) {
        if let Ok(lines) = super::read::lines(file) {
            // Consumes the iterator, returns an (Optional) String
            for line in lines.flatten() {
                debug!("{}", line);
            }
        }
    }
}

pub mod read {
    use crate::error::DggrsError;
    use crate::error::dggrid::DggridError;
    use crate::types::Point;
    use crate::types::{Region, Zone, ZoneId};
    use core::f64;
    use std::collections::{BTreeMap, HashMap};
    use std::fs;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;
    /// Used to parse the AIGEN output file of DGGRID that always contains the center and the region
    pub fn parse_aigen_to_zones_map(s: &str) -> Result<BTreeMap<ZoneId, Zone>, DggrsError> {
        let mut out = BTreeMap::new();

        struct AigenZoneRegionCenter {
            id: ZoneId,
            xy: (f64, f64),
            vec_xy: Vec<(f64, f64)>,
        }
        let mut cur: Option<AigenZoneRegionCenter> = None;

        for line in s.lines() {
            // Each line of the AIGEN file is split at the whitespace
            let parts: Vec<&str> = line.split_whitespace().collect();
            // NOTE:
            // Here we match three options:
            //      1. The ID with x and y for the center
            //      2. x and y for the polygon (multiple lines)
            //      3. "END" to signify the end of the polygon geometry
            // first step is to take the parts from above and consider the slice only.
            match parts.as_slice() {
                // header: <ID> <cx> <cy>  thats the center coordinates after the ID
                [id_str, cx, cy] => {
                    cur = Some(AigenZoneRegionCenter {
                        id: ZoneId::new_hex(id_str)?,
                        xy: (cx.parse()?, cy.parse()?),
                        vec_xy: Vec::new(),
                    });
                }
                // vertex line: <x> <y>
                [x, y] => {
                    if let Some(z) = cur.as_mut() {
                        z.vec_xy.push((x.parse()?, y.parse()?));
                    }
                }
                // END marker
                ["END"] => {
                    // NOTE:
                    // When END is reachted the vec_xy for the polygon is finished.
                    // this also mean that the second END at the end of the file
                    // cannot take anything and is None.
                    if let Some(z) = cur.take() {
                        // NOTE:
                        // there are options to control the cell_output_type and
                        // point_output_type in DGGRID, maybe we can avoid generating everything
                        // so we do not have to parse it also.
                        let pnt = Some(Point::new(z.xy.1, z.xy.0));

                        let poly = if z.vec_xy.len() >= 2 {
                            let region_points: Vec<Point> = z
                                .vec_xy
                                .iter()
                                .map(|(x, y)| Point::new(*y, *x))
                                .collect();
                            Some(Region::new(region_points))
                        } else {
                            None
                        };

                        out.insert(
                            z.id.clone(),
                            Zone {
                                id: z.id,
                                center: pnt,
                                region: poly,
                                children: None,
                                neighbors: None,
                                vertex_count: None,
                                area_sqm: None,
                            },
                        );
                    }
                }
                // ignore blanks or comments
                [] => {}
                _ => {
                    // You can choose to return an error here
                }
            }
        }
        Ok(out)
    }

    /// Used to parse the text output from DGGRID for children and neighbors
    pub fn parse_id_list(s: &str) -> Result<HashMap<ZoneId, Vec<ZoneId>>, DggrsError> {
        let mut map: HashMap<ZoneId, Vec<ZoneId>> = HashMap::new();

        for (lineno, line) in s.lines().enumerate() {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }
            let key = ZoneId::new_hex(parts[0])?;

            let vals = parts
                .iter()
                .skip(1)
                .map(|t| ZoneId::new_hex(t))
                .collect::<Result<Vec<_>, _>>()?;

            if map.insert(key, vals).is_some() {
                return Err(DggrsError::Dggrid(DggridError::Malformed {
                    msg: format!("duplicate key on line {}", lineno + 1),
                }));
            }
        }
        Ok(map)
    }

    // Read aigen file produced by DGGRID
    // Todo: this is inefficient, use the read_lines function as in print_file
    // https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
    pub fn file(path: &Path) -> Result<String, DggridError> {
        fs::read_to_string(path).map_err(|e| DggridError::FileRead {
            path: path.display().to_string(),
            source: e,
        })
    }

    pub fn lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}

pub mod output {
    use crate::api::DggrsApiConfig;
    use crate::error::DggrsError;
    use crate::types::{ZoneId, Zones};
    use geo::GeodesicArea;
    use itertools::Itertools;
    use std::collections::HashMap;
    use std::path::Path;

    pub fn ingest(
        aigen_path: &Path,
        children_path: &Path,
        neighbors_path: &Path,
        conf: &DggrsApiConfig,
    ) -> Result<Zones, DggrsError> {
        // the default output
        let aigen_text = super::read::file(&aigen_path)?;
        let mut zones_map = super::read::parse_aigen_to_zones_map(&aigen_text)?;

        // children
        let mut children_map: HashMap<ZoneId, Vec<ZoneId>> = if conf.children {
            let children_text = super::read::file(&children_path)?;
            super::read::parse_id_list(&children_text)?
        } else {
            HashMap::new()
        };

        // neighbors
        let mut neighbors_map: HashMap<ZoneId, Vec<ZoneId>> = if conf.neighbors {
            let neighbors_text = super::read::file(&neighbors_path)?;
            super::read::parse_id_list(&neighbors_text)?
        } else {
            HashMap::new()
        };

        // Assemble outputs
        for (id, z) in zones_map.iter_mut() {
            // attach children
            if let Some(v) = children_map.remove(id) {
                z.children = Some(v);
            }

            // attach neighbors
            if let Some(v) = neighbors_map.remove(id) {
                z.neighbors = Some(v);
            }

            // vertex count
            // WARN: actually this is not counting the vertices, it is counting the
            // corners/nodes/edges of of shapes like triangle, rhombus, pentagon or hexagons
            if conf.vertex_count {
                if let Some(ref poly) = z.region {
                    z.vertex_count = Some(super::helper::corner_count_convex(poly));
                }
            }

            // compute area
            if conf.area_sqm {
                if let Some(ref poly) = z.region {
                    // NOTE:
                    // It may be a good idea to wrap geodesic_area_unsigned into
                    // a separate extension trait, so that we don't use a different
                    // calculation elsewhere by accident.
                    z.area_sqm = Some(poly.to_geo_polygon().geodesic_area_unsigned());
                }
            }

            // drop geometry not requested
            if !conf.region {
                z.region = None;
            }

            if !conf.center {
                z.center = None;
            }
        }
        Ok(Zones {
            zones: zones_map
                .into_values()
                .sorted_by(|a, b| b.id.cmp(&a.id))
                .collect(),
        })
    }
}

pub mod helper {
    use crate::types::Region;
    use geo::CoordsIter;
    use geo::prelude::ConvexHull;
    pub fn corner_count_convex(poly: &Region) -> u32 {
        let hull = poly.to_geo_polygon().convex_hull();
        // coords_count() includes the closing vertex => subtract 1
        (hull.exterior().coords_count() as u32).saturating_sub(1)
    }
}

use std::fs;
use std::path::Path;
pub fn cleanup(
    meta_path: &Path,
    aigen_path: &Path,
    children_path: &Path,
    neighbor_path: &Path,
    bbox_path: &Path,
    input_path: &Path,
) {
    let _ = fs::remove_file(meta_path);
    let _ = fs::remove_file(aigen_path);
    let _ = fs::remove_file(children_path);
    let _ = fs::remove_file(neighbor_path);
    let _ = fs::remove_file(bbox_path);
    let _ = fs::remove_file(input_path);
}
