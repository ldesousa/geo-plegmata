use crate::models::common::{CellGEO, CellsGEO};
use crate::models::dggrid::{CellID, IdArray};
use crate::ports::dggrs::DggrsPort;
use core::f64;
use geo::geometry::{LineString, Point, Polygon};
use rand::distributions::{Alphanumeric, DistString};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::debug;

const DENSIFICATION: u8 = 50; // DGGRID option
const CLIP_CELL_DENSIFICATION: u8 = 50; // DGGRID option

fn print_file(file: PathBuf) {
    if let Ok(lines) = read_lines(file) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            debug!("{}", line);
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

//pub fn polygon() -> Cells {}
//pub fn point() -> Cells {}
fn dggrid_setup(workdir: &PathBuf) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
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

fn dggrid_execute(dggrid_path: &PathBuf, meta_path: &PathBuf) {
    let _ = Command::new(&dggrid_path).arg(&meta_path).output();
}

fn dggrid_parse(
    aigen_path: &PathBuf,
    children_path: &PathBuf,
    neighbor_path: &PathBuf,
    dggs_type: &String,
    dggs_res_spec: &u8,
) -> CellsGEO {
    let aigen_data = read_file(&aigen_path);
    let mut result = parse_aigen(&aigen_data, &dggs_type, &dggs_res_spec);
    let children_data = read_file(&children_path);
    let children = parse_children(&children_data, &dggs_res_spec);
    assign_field(&mut result, children, "children");

    let neighbor_data = read_file(&neighbor_path);
    let neighbors = parse_neighbors(&neighbor_data, &dggs_res_spec);
    assign_field(&mut result, neighbors, "neighbors");
    result
}

fn dggrid_cleanup(
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

fn dggrid_metafile(
    metafile: &PathBuf,
    dggs_type: &String,
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

    if *dggs_type == String::from("ISEA3H") {
        writeln!(file, "dggs_type {}", dggs_type)?;
        writeln!(file, "dggs_aperture 3")?;
        writeln!(file, "output_address_type Z3")?;
    } else if *dggs_type == String::from("IGEO7") {
        writeln!(file, "dggs_type {}", dggs_type)?;
        writeln!(file, "dggs_aperture 7")?;
        writeln!(file, "output_address_type Z7")?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unsupported DGGS Type",
        ));
    };

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

/// Read aigen file produced by DGGRID
/// Todo: this is inefficient, use the read_lines function as in print_file
/// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
fn read_file(path: &Path) -> String {
    let data = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "Unable to read file {}: {}",
            path.to_str().unwrap_or("unknown"),
            e
        )
    });
    data
}

fn parse_children(data: &String, dggs_res_spec: &u8) -> Vec<IdArray> {
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
fn parse_neighbors(data: &String, dggs_res_spec: &u8) -> Vec<IdArray> {
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

fn parse_aigen(data: &String, dggs_type: &String, dggs_res_spec: &u8) -> CellsGEO {
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
            let id_str = if dggs_type == "ISEA3H" {
                format!("{:02}{}", dggs_res_spec, line_parts[0])
            } else if dggs_type == "IGEO7" {
                format!("{:02}{}", dggs_res_spec, line_parts[0])
            } else {
                line_parts[0].to_string()
            };
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

pub struct DggridAdapter {
    executable: PathBuf,
    workdir: PathBuf,
}

impl DggridAdapter {
    pub fn new(executable: PathBuf, workdir: PathBuf) -> Self {
        DggridAdapter {
            executable,
            workdir,
        }
    }
}

impl DggrsPort for DggridAdapter {
    fn whole_earth(
        &self,
        dggs_type: String,
        dggs_res_spec: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> CellsGEO {
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, _input_path) =
            dggrid_setup(&self.workdir);

        let _ = dggrid_metafile(
            &meta_path,
            &dggs_type,
            &dggs_res_spec,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            densify,
        );

        if let Some(bbox) = &bbox {
            let _ = bbox_to_aigen(bbox, &bbox_path);

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

        print_file(meta_path.clone());
        dggrid_execute(&self.executable, &meta_path);
        let result = dggrid_parse(
            &aigen_path,
            &children_path,
            &neighbor_path,
            &dggs_type,
            &dggs_res_spec,
        );
        dggrid_cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
        );
        result
    }

    fn from_point(
        &self,
        dggs_type: String,
        dggs_res_spec: u8,
        point: Point,
        densify: bool,
    ) -> CellsGEO {
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            dggrid_setup(&self.workdir);

        let _ = dggrid_metafile(
            &meta_path,
            &dggs_type,
            &dggs_res_spec,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            densify,
        );

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

        print_file(meta_path.clone());
        dggrid_execute(&self.executable, &meta_path);
        let result = dggrid_parse(
            &aigen_path,
            &children_path,
            &neighbor_path,
            &dggs_type,
            &dggs_res_spec,
        );
        dggrid_cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
        );
        let _ = fs::remove_file(&input_path);
        result
    }
    fn coarse_cells(
        &self,
        dggs_type: String,
        dggs_res_spec: u8,
        clip_cell_addresses: String, // ToDo: needs validation function
        // clip_cell_res: u8,
        densify: bool,
    ) -> CellsGEO {
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, _input_path) =
            dggrid_setup(&self.workdir);

        let _ = dggrid_metafile(
            &meta_path,
            &dggs_type,
            &dggs_res_spec,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            densify,
        );

        // Append to metafile format
        let mut meta_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&meta_path)
            .expect("cannot open file");

        let clip_cell_res = extract_res_from_cellid(&clip_cell_addresses, &dggs_type).unwrap();

        let clip_cell_address = if dggs_type == "ISEA3H" {
            &clip_cell_addresses[2..] // strip first two characters
        } else if dggs_type == "IGEO7" {
            &clip_cell_addresses[2..] // ToDo: this needs replacementi
        } else {
            &clip_cell_addresses
        };

        let _ = writeln!(meta_file, "clip_subset_type COARSE_CELLS");
        let _ = writeln!(meta_file, "clip_cell_res {:?}", clip_cell_res);
        let _ = writeln!(
            meta_file,
            "clip_cell_densification {}",
            CLIP_CELL_DENSIFICATION
        );
        let _ = writeln!(meta_file, "clip_cell_addresses \"{}\"", clip_cell_address);
        if &dggs_type == "ISEA3H" {
            let _ = writeln!(meta_file, "input_address_type Z3");
        } else if dggs_type == "IGEO7" {
            let _ = writeln!(meta_file, "input_address_type Z7");
        };
        print_file(meta_path.clone());
        dggrid_execute(&self.executable, &meta_path);
        let result = dggrid_parse(
            &aigen_path,
            &children_path,
            &neighbor_path,
            &dggs_type,
            &dggs_res_spec,
        );
        dggrid_cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
        );
        result
    }
    fn single_zone(
        &self,
        dggs_type: String,
        zone_id: String, // ToDo: needs validation function
        densify: bool,
    ) -> CellsGEO {
        let (meta_path, aigen_path, children_path, neighbor_path, bbox_path, input_path) =
            dggrid_setup(&self.workdir);

        let clip_cell_res = extract_res_from_cellid(&zone_id, &dggs_type).unwrap();
        let dggs_res_spec = clip_cell_res;
        let _ = dggrid_metafile(
            &meta_path,
            &dggs_type,
            &dggs_res_spec,
            &aigen_path.with_extension(""),
            &children_path.with_extension(""),
            &neighbor_path.with_extension(""),
            densify,
        );

        // Append to metafile format
        let mut meta_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&meta_path)
            .expect("cannot open file");

        let zone = if dggs_type == "ISEA3H" {
            &zone_id[2..] // strip first two characters
        } else if dggs_type == "IGEO7" {
            &zone_id[2..] // ToDo: this needs replacementi
        } else {
            &zone_id
        };

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
        if &dggs_type == "ISEA3H" {
            let _ = writeln!(meta_file, "input_address_type Z3");
        } else if dggs_type == "IGEO7" {
            let _ = writeln!(meta_file, "input_address_type Z7");
        };
        print_file(meta_path.clone());
        dggrid_execute(&self.executable, &meta_path);
        let result = dggrid_parse(
            &aigen_path,
            &children_path,
            &neighbor_path,
            &dggs_type,
            &dggs_res_spec,
        );
        dggrid_cleanup(
            &meta_path,
            &aigen_path,
            &children_path,
            &neighbor_path,
            &bbox_path,
        );
        result
    }
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
