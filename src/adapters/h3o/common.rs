use geo::{Coord, LineString, Point, Polygon};
use h3o::{Boundary, CellIndex, LatLng, Resolution};

pub fn res(level: u8) -> Resolution {
    Resolution::try_from(level).unwrap_or_else(|_| panic!("Invalid H3 resolution: {}", level))
}

pub fn boundary_to_polygon(boundary: &Boundary) -> Polygon<f64> {
    let mut coords: Vec<Coord<f64>> = boundary
        .iter()
        .map(|latlng| Coord {
            x: latlng.lng(),
            y: latlng.lat(),
        })
        .collect();

    // Ensure the ring is closed
    if coords.first() != coords.last() {
        if let Some(first) = coords.first().copied() {
            coords.push(first);
        }
    }

    Polygon::new(LineString::from(coords), vec![])
}

pub fn children_to_strings(iter: impl Iterator<Item = CellIndex>) -> Vec<String> {
    iter.map(|cell| cell.to_string()).collect()
}

pub fn ring_to_strings(iter: impl Iterator<Item = Option<CellIndex>>) -> Vec<String> {
    iter.filter_map(|opt| opt.map(|cell| cell.to_string()))
        .collect()
}

pub fn latlng_to_point(latlng: LatLng) -> Point {
    Point::new(latlng.lng(), latlng.lat())
}
