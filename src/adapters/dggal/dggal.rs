use crate::ports::dggrs::DggrsPort;

pub struct DggalImpl;

impl DggrsPort for DggalImpl {
    fn whole_earth(
        &self,
        dggs_type: String,
        dggs_res_spec: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> CellsGEO {
        // call dggal here and return CellsGEO
    }
    fn from_point(
        &self,
        dggs_type: String,
        dggs_res_spec: u8,
        point: Point,
        densify: bool,
    ) -> CellsGEO {
        // etc.
    }
    fn coarse_cells(&self, ...) -> CellsGEO {
        // ...
    }
    fn single_zone(&self, ...) -> CellsGEO {
        // ...
    }
}
