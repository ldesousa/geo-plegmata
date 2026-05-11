use geoplegma::adapters::dggal::grids::DggalImpl;
use geoplegma::adapters::dggrid::igeo7::Igeo7Impl;
use geoplegma::adapters::dggrid::isea3h::Isea3hImpl;
use geoplegma::adapters::h3o::h3::H3Impl;
use geoplegma::api::{DggrsApi, DggrsApiConfig};
use geoplegma::types::{DggrsUid, RefinementLevel, Point};


#[test]
fn h3_parent_from_zone_contains_child_zone() {
    let adapter = H3Impl::default();
    test_parent_from_zone_contains_child_zone(&adapter);
}

#[test]
fn dggal_parent_from_zone_contains_child_zone() {
    let adapter = DggalImpl::new(DggrsUid::ISEA3HDGGAL);
    test_parent_from_zone_contains_child_zone(&adapter);
}

#[test]
fn igeo7_parent_from_zone_contains_child_zone() {
    let adapter = Igeo7Impl::default();
    test_parent_from_zone_contains_child_zone(&adapter);
}

#[test]
fn isea3h_parent_from_zone_contains_child_zone() {
    let adapter = Isea3hImpl::default();
    test_parent_from_zone_contains_child_zone(&adapter);
}

fn test_parent_from_zone_contains_child_zone<T: DggrsApi>(adapter: &T) {
    let point = Point::new(52.98, 9.06);
    let base_config = DggrsApiConfig {
        area_sqm: false,
        densify: false,
        center: false,
        region: false,
        children: false,
        neighbors: false,
        vertex_count: false,
    };
    let parent_config = DggrsApiConfig {
        children: true,
        ..base_config
    };

    for rf in 1..15 {
        let child_level = RefinementLevel::new(rf).unwrap();
        let child_zone_result = adapter
            .zone_from_point(child_level, point, Some(base_config))
            .unwrap()
            .zones;

        let child_zone = child_zone_result.first().map(|zone| zone.id.clone()).unwrap();

        let parent_zone = adapter
            .primary_parent_from_zone(child_zone.clone(), Some(parent_config))
            .unwrap()
            .zones
            .first()
            .unwrap()
            .clone();

        assert!(
            parent_zone.children.unwrap().contains(&child_zone),
            "Parent zone does not contain the queried child zone"
        );
    }
}
