use crate::adapters::h3o::common::{
    boundary_to_polygon, children_to_strings, latlng_to_point, res, ring_to_strings,
};
use crate::models::common::Zones;
use crate::ports::dggrs::DggrsPort;
use geo::Point;
use h3o::LatLng;

pub struct H3Impl;

impl DggrsPort for H3Impl {
    fn zones_from_bbox(
        &self,
        _dggs_res_spec: u8,
        _densify: bool,
        _bbox: Option<Vec<Vec<f64>>>,
    ) -> Zones {
        unimplemented!("whole earth is not yet implemented for h3");
    }
    fn zone_from_point(&self, dggs_res_spec: u8, point: Point, _densify: bool) -> Zones {
        let coord = LatLng::new(point.x(), point.y()).expect("valid coord");

        let cell = coord.to_cell(res(dggs_res_spec));
        println!("Cell ID {}", cell);

        let center = latlng_to_point(LatLng::from(cell));
        println!("Cell CENTER {:?}", center);

        let children = children_to_strings(cell.children(res(dggs_res_spec + 1)));
        println!("Cell CHILDREN {:?}", children);

        let boundary = cell.boundary();
        let polygon = boundary_to_polygon(&boundary);
        println!("Cell POLYGON {:?}", polygon);

        // grid_ring_fast(1) gives the direct neighbors
        let neighbors = ring_to_strings(cell.grid_ring_fast(1));
        println!("Cell NEIGHBORS {:?}", neighbors);

        todo!("output to Zones");
    }
    fn zones_from_parent(
        &self,
        _dggs_res_spec: u8,
        _clip_cell_addresses: String, // ToDo: needs validation function
        _densify: bool,
    ) -> Zones {
        unimplemented!("coarse_cells is not yet implemented for h3");
    }
    fn zone_from_id(
        &self,
        _zone_id: String, // ToDo: needs validation function
        _densify: bool,
    ) -> Zones {
        unimplemented!("single_zone is not yet implemented for h3");
    }
}
