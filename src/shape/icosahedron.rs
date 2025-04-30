use crate::models::common::PositionGeo;
use crate::utils::math::{cos, sin, to_deg, to_rad};
use crate::{traits::polyhedron::Polyhedron, utils::math::pow};

use crate::traits::polyhedron::{ArcLengths, UnitVectors, Vertexes};

/// Spherical Constant
/// Spherical distance in degrees (g) for the icosahedron, from center of polygon face to any of its vertices on the globe.
pub const SPHERICAL_DISTANCE: f64 = 37.37736814; // g

/// Spherical Constant
/// Plane angle in degrees (θ), between the radius vector (R´) to the center and adjacent edge of plane polygon.
pub const PLANE_ANGLE: f64 = 30.0; // θ

pub const GOLDEN_RATIO: f64 = 1.618;

#[derive(Default, Debug)]
pub struct Icosahedron {
    // pub triangle_uvs: UnitVectors,
    // pub triangle_arcs: ArcLengths,
    // pub triangle_vertexes: Vertexes,
}

impl Polyhedron for Icosahedron {
    // fn golden_ratio(&self) -> f64 {
    //     GOLDEN_RATIO
    // }
    // fn plane_angle(&self) -> f64 {
    //     PLANE_ANGLE
    // }
    // fn spherical_distance(&self) -> f64 {
    //     SPHERICAL_DISTANCE
    // }
    // fn new(self)  {
    //     let mut ico = Self {
    //         triangle_uvs: Default::default(),
    //         triangle_arcs: Default::default(),
    //         triangle_vertexes: Default::default(),
    //     };
    //     ico.unit_vectors();
    //     ico.triangle_arc_lengths();
    //     ico.planar_vertexes();
    //     ico;
    // }
    fn get_planar_vertexes(&self) -> Vertexes {
        let a = [0.0, 0.0];
        let b = [0.8944, 0.0];
        let c = [0.2764, -0.8507];

        Vertexes { a, b, c }
    }
    fn get_triangle_unit_vectors(&self) -> UnitVectors {
        let aux = 1.0 / f64::sqrt(1.0 + pow(self::GOLDEN_RATIO, 2));
        let aux1 = self::GOLDEN_RATIO / f64::sqrt(1.0 + pow(self::GOLDEN_RATIO, 2));

        let a = [0.0, aux, aux1];
        let b = [aux, aux1, 0.0];
        let c = [aux1, 0.0, aux];

        UnitVectors { a, b, c }
    }

    // to 90 degrees right triangle
    fn get_triangle_arc_lengths(&self, p: PositionGeo) -> ArcLengths {
        let uvs = self.get_triangle_unit_vectors();
        let dot_ab = 0.0;
        let ab = f64::acos(dot_ab);
        let bc = f64::acos(uvs.b[0] * uvs.c[0] + uvs.b[1] * uvs.c[1] + uvs.b[2] * uvs.c[2]);
        let ac = f64::acos(uvs.a[0] * uvs.c[0] + uvs.a[1] * uvs.c[1] + uvs.a[2] * uvs.c[2]);

        let lat = to_rad(p.lat);
        let lon = to_rad(p.lon);
        // calculate unit vectors for point P
        let uv_px = cos(lat) * cos(lon);
        let uv_py = cos(lat) * sin(lon);
        let uv_pz = sin(lat);

        let ap = f64::acos(uvs.a[0] * uv_px + uvs.a[1] * uv_py + uvs.a[2] * uv_pz); //f64::acos(uvs.a[0] * uvp[0] + uvs.a[1] * uvp[1] + uvs.a[2] * uvp[2]);
        let bp = f64::acos(uvs.b[0] * uv_px + uvs.b[1] * uv_py + uvs.b[2] * uv_pz);
        let cp = f64::acos(uvs.c[0] * uv_px + uvs.c[1] * uv_py + uvs.c[2] * uv_pz);

        println!("{:?}", [ab, bc, ac, ap, bp, cp]);
        ArcLengths {
            ab,
            bc,
            ac,
            ap,
            bp,
            cp,
        }
    }
    // fn get_center_faces_icosahedron() -> Vec<Position> {
    //     let e: f64 = 90.0 - consts::SPHERICAL_DISTANCE; // E in degrees
    //     let f: f64 = 10.81231696; // F in degrees
    //     // let lats: [f64; 20] = [
    //     //     e, e, e, e, e, f, f, f, f, f, -f, -f, -f, -f, -f, -e, -e, -e, -e, -e,
    //     // ];
    //     // let lons: [f64; 20] = [
    //     //     -144.0, -72.0, 0.0, 72.0, 144.0, -144.0, -72.0, 0.0, 72.0, 144.0, -108.0, -36.0, 36.0,
    //     //     108.0, 180.0, -108.0, -36.0, 36.0, 108.0, 180.0,
    //     // ];

    //     vec![
    //         Position {
    //             lat: e,
    //             lon: -144.0,
    //         },
    //         Position { lat: e, lon: -72.0 },
    //         Position { lat: e, lon: 0.0 },
    //         Position { lat: e, lon: 72.0 },
    //         Position { lat: e, lon: 144.0 },
    //         Position {
    //             lat: f,
    //             lon: -144.0,
    //         },
    //         Position { lat: f, lon: -72.0 },
    //         Position { lat: f, lon: 0.0 },
    //         Position { lat: f, lon: 72.0 },
    //         Position { lat: f, lon: 144.0 },
    //         Position {
    //             lat: -f,
    //             lon: -108.0,
    //         },
    //         Position {
    //             lat: -f,
    //             lon: -36.0,
    //         },
    //         Position { lat: -f, lon: 36.0 },
    //         Position {
    //             lat: -f,
    //             lon: 108.0,
    //         },
    //         Position {
    //             lat: -f,
    //             lon: 180.0,
    //         },
    //         Position {
    //             lat: -e,
    //             lon: -108.0,
    //         },
    //         Position {
    //             lat: -e,
    //             lon: -36.0,
    //         },
    //         Position { lat: -e, lon: 36.0 },
    //         Position {
    //             lat: -e,
    //             lon: 108.0,
    //         },
    //         Position {
    //             lat: -e,
    //             lon: 180.0,
    //         },
    //     ]
    // }
}
