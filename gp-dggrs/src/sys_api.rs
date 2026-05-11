//use api::error::DggrsError;
use geo::Point;
use geoplegma::types::RefinementLevel; //, Zones};

pub trait DggrsSysApi {
    const APERTURE: u32;

    fn zone_from_point(
        &self,
        _refinement_level: RefinementLevel,
        _point: Point,
        //config: Option<DggrsApiConfig>,
    ) -> u64 {
        return 0;
    }

    fn get_children() {}
}
