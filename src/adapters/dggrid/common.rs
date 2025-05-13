// Copyright 2025 contributors to the GeoPlegmata project.
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::models::common::{CellGEO, CellsGEO};
use crate::models::dggrid::CellID;
use crate::models::dggrid::IdArray;
use core::f64;
use geo::geometry::{LineString, Point, Polygon};
use rand::distributions::{Alphanumeric, DistString};
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::debug;

pub const DENSIFICATION: u8 = 50; // DGGRID option

pub fn dggrid_setup(workdir: &PathBuf) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
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

pub fn dggrid_metafile(
    metafile: &PathBuf,
    dggs_res_spec: &u8,
    cell_output_file_name: &PathBuf,
    children_output_file_name: &PathBuf,
    neighbor_output_file_name: &PathBuf,
    densify: bool,
) -> io::Result<()> {
    debug!("Writing to {:?}", metafile);
    let mut file = fs::File::create(metafile)?;
    writeln!(file, "longitude_wrap_mode UNWRAP_EAST")?;
    writeln!(file, "cell_output_type AIGEN")?;
    writeln!(file, "unwrap_points FALSE")?;
    writeln!(file, "output_cell_label_type OUTPUT_ADDRESS_TYPE")?;
    writeln!(file, "precision 9")?;
    writeln!(file, "dggs_res_spec {}", dggs_res_spec)?;

    writeln!(
        file,
        "cell_output_file_name {}",
        cell_output_file_name.to_string_lossy().into_owned()
    )?;

    writeln!(file, "neighbor_output_type TEXT")?;
    writeln!(
        file,
        "neighbor_output_file_name {}",
        neighbor_output_file_name.to_string_lossy().into_owned()
    )?;
    writeln!(file, "children_output_type TEXT")?;
    writeln!(
        file,
        "children_output_file_name {}",
        children_output_file_name.to_string_lossy().into_owned()
    )?;

    if densify == true {
        writeln!(file, "densification {}", DENSIFICATION)?;
    }

    Ok(())
}
pub fn dggrid_execute(dggrid_path: &PathBuf, meta_path: &PathBuf) {
    let _ = Command::new(&dggrid_path).arg(&meta_path).output();
}

pub fn dggrid_parse(
    aigen_path: &PathBuf,
    children_path: &PathBuf,
    neighbor_path: &PathBuf,
    dggs_res_spec: &u8,
) -> CellsGEO {
    let aigen_data = read_file(&aigen_path);
    let mut result = parse_aigen(&aigen_data, &dggs_res_spec);
    let children_data = read_file(&children_path);
    let children = parse_children(&children_data, &dggs_res_spec);
    assign_field(&mut result, children, "children");

    let neighbor_data = read_file(&neighbor_path);
    let neighbors = parse_neighbors(&neighbor_data, &dggs_res_spec);
    assign_field(&mut result, neighbors, "neighbors");
    result
}

pub fn parse_aigen(data: &String, dggs_res_spec: &u8) -> CellsGEO {
    let mut cell_id = CellID::default();
    let mut cells_geo = CellsGEO { cells: Vec::new() };

    let mut raw_coords: Vec<(f64, f64)> = vec![];
    let mut ply: Polygon;
    let mut pnt = Point::new(0.0, 0.0);
    let mut v_count = 0u32;

    // loop over the entire AIGEN file
    for line in data.lines() {
        // println!("{:?}", line);
        let line_parts: Vec<&str> = line.split_whitespace().collect();
        // The first line of each hexagon is always 3 strings, the first is the ID and the
        // second two are the center point

        if line_parts.len() == 3 {
            // For ISEA3H prepend zero-padded dggs_res_spec to the ID
            let id_str = format!("{:02}{}", dggs_res_spec, line_parts[0]);
            cell_id = CellID::new(&id_str).expect("Cannot accept this id");
            pnt = Point::new(
                line_parts[1]
                    .parse::<f64>()
                    .expect("cannot parse floating point number"),
                line_parts[2]
                    .parse::<f64>()
                    .expect("cannot parse floating point number"),
            );
        // these are coordinate pairs for the region
        } else if line_parts.len() == 2 {
            v_count += 1;
            raw_coords.push((
                line_parts[0]
                    .parse::<f64>()
                    .expect("cannot parse floating point number"),
                line_parts[1]
                    .parse::<f64>()
                    .expect("cannot parse floating point number"),
            ))
        // if it just 1 part AND it is END AND if the vertex count is larger than 1
        } else if line_parts.len() == 1 && line_parts[0] == "END" && v_count > 1 {
            ply = Polygon::new(LineString::from(raw_coords.clone()), vec![]);

            let cell_geo = CellGEO {
                id: cell_id.clone(),
                region: ply,
                center: pnt,
                vertex_count: v_count - 1,
                children: None,
                neighbors: None,
            };
            cells_geo.cells.push(cell_geo);

            // reset
            raw_coords.clear();
            v_count = 0;
        }
    }
    cells_geo
}
pub fn dggrid_cleanup(
    meta_path: &PathBuf,
    aigen_path: &PathBuf,
    children_path: &PathBuf,
    neighbor_path: &PathBuf,
    bbox_path: &PathBuf,
) {
    let _ = fs::remove_file(meta_path);
    let _ = fs::remove_file(aigen_path);
    let _ = fs::remove_file(children_path);
    let _ = fs::remove_file(neighbor_path);
    let _ = fs::remove_file(bbox_path);
}

pub fn parse_children(data: &String, dggs_res_spec: &u8) -> Vec<IdArray> {
    data.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }

            let id = Some(format!("{:02}{}", dggs_res_spec, parts[0]));
            let arr = parts
                .iter()
                .skip(1)
                .map(|s| format!("{:02}{}", dggs_res_spec + 1, s))
                .collect();

            Some(IdArray { id, arr: Some(arr) })
        })
        .collect()
}
pub fn parse_neighbors(data: &String, dggs_res_spec: &u8) -> Vec<IdArray> {
    data.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }

            let id = Some(format!("{:02}{}", dggs_res_spec, parts[0]));
            let arr = parts
                .iter()
                .skip(1)
                .map(|s| format!("{:02}{}", dggs_res_spec, s))
                .collect();

            Some(IdArray { id, arr: Some(arr) })
        })
        .collect()
}

pub fn assign_field(cells_geo: &mut CellsGEO, data: Vec<IdArray>, field: &str) {
    for item in data {
        if let Some(ref id_str) = item.id {
            if let Some(cell) = cells_geo
                .cells
                .iter_mut()
                .find(|c| c.id.to_string() == *id_str)
            {
                match field {
                    "children" => cell.children = item.arr.clone(),
                    "neighbors" => cell.neighbors = item.arr.clone(),
                    _ => panic!("Unknown field: {}", field),
                }
            }
        }
    }
}

pub fn print_file(file: PathBuf) {
    if let Ok(lines) = read_lines(file) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            debug!("{}", line);
        }
    }
}

/// Read aigen file produced by DGGRID
/// Todo: this is inefficient, use the read_lines function as in print_file
/// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
pub fn read_file(path: &Path) -> String {
    let data = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "Unable to read file {}: {}",
            path.to_str().unwrap_or("unknown"),
            e
        )
    });
    data
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn bbox_to_aigen(bbox: &Vec<Vec<f64>>, bboxfile: &PathBuf) -> io::Result<()> {
    if bbox.len() != 2 || bbox[0].len() != 2 || bbox[1].len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid bounding box format",
        ));
    }

    let (minx, miny) = (bbox[0][0], bbox[0][1]);
    let (maxx, maxy) = (bbox[1][0], bbox[1][1]);

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
    writeln!(file, "1 {:.6} {:.6}", center_x, center_y)?;

    for (x, y) in &vertices {
        writeln!(file, "{:.6} {:.6}", x, y)?;
    }

    writeln!(file, "END")?;
    writeln!(file, "END")?;

    Ok(())
}
