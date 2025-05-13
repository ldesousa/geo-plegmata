use crate::adapters::dggrid::DggridAdapter;
use crate::models::common::CellsGEO;
use crate::ports::dggrs::DggrsPort;
use geo::Point;
use std::path::PathBuf;

pub struct DggridService(DggridAdapter);

impl DggridService {
    pub fn default() -> Self {
        Self(DggridAdapter::new(
            PathBuf::from("dggrid"),
            PathBuf::from("/dev/shm"),
        ))
    }

    pub fn new(excecutable: PathBuf, workdir: PathBuf) -> Self {
        Self(DggridAdapter::new(excecutable, workdir))
    }

    pub fn whole_earth(
        &self,
        dggs_type: String,
        dggs_res_spec: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> CellsGEO {
        self.0.whole_earth(dggs_type, dggs_res_spec, densify, bbox)
    }

    pub fn from_point(
        &self,
        dggs_type: String,
        dggs_res_spec: u8,
        point: Point,
        densify: bool,
    ) -> CellsGEO {
        self.0.from_point(dggs_type, dggs_res_spec, point, densify)
    }

    pub fn coarse_cells(
        &self,
        dggs_type: String,
        dggs_res_spec: u8,
        clip_cell_addresses: String,
        // clip_cell_res: u8,
        densify: bool,
    ) -> CellsGEO {
        self.0.coarse_cells(
            dggs_type,
            dggs_res_spec,
            clip_cell_addresses,
            // clip_cell_res,
            densify,
        )
    }
    pub fn single_zone(&self, dggs_type: String, zone_id: String, densify: bool) -> CellsGEO {
        self.0.single_zone(dggs_type, zone_id, densify)
    }
}
