use crate::{models::common::Position, traits::polyhedron::Polyhedron, utils::math::pow};

pub mod consts {
    /// Spherical Constant
    /// Spherical distance in degrees (g) for the icosahedron, from center of polygon face to any of its vertices on the globe.
    pub const SPHERICAL_DISTANCE: f64 = 37.37736814; // g

    /// Spherical Constant
    /// Plane angle in degrees (θ), between the radius vector (R´) to the center and adjacent edge of plane polygon.
    pub const PLANE_ANGLE: f64 = 30.0; // θ

    pub const GOLDEN_RATIO: f64 = 1.618;
}
#[derive(Default, Debug)]
pub struct UnitVectors {
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
}
#[derive(Default, Debug)]
pub struct ArcLengths {
    pub ab: f64,
    pub bc: f64,
    pub ac: f64,
    pub ap: f64,
    pub bp: f64,
    pub cp: f64,
}
#[derive(Default, Debug)]
pub struct Vertexes {
    pub a: [f64; 2],
    pub b: [f64; 2],
    pub c: [f64; 2],
}
#[derive(Default, Debug)]
pub struct Icosahedron {
    pub triangle_uvs: UnitVectors,
    pub triangle_arcs: ArcLengths,
    pub triangle_vertexes: Vertexes,
}

impl Polyhedron for Icosahedron {
    fn new(&mut self)  {
        let mut ico = Self {
            triangle_uvs: Default::default(),
            triangle_arcs: Default::default(),
            triangle_vertexes: Default::default(),
        };
        ico.unit_vectors();
        ico.triangle_arc_lengths();
        ico.planar_vertexes();
        ico;
    }
    fn planar_vertexes(&mut self)  {
        self.triangle_vertexes.a = vec![0.0, 0.0]
            .try_into()
            .expect("Expected exactly 3 elements");
        self.triangle_vertexes.b = vec![0.8944, 0.0]
            .try_into()
            .expect("Expected exactly 3 elements");
        self.triangle_vertexes.c = vec![0.2764, -0.8507]
            .try_into()
            .expect("Expected exactly 3 elements");
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
    fn unit_vectors(&mut self)   {
        let aux = 1.0 / f64::sqrt(1.0 + pow(consts::GOLDEN_RATIO, 2));
        let aux1 = consts::GOLDEN_RATIO / f64::sqrt(1.0 + pow(consts::GOLDEN_RATIO, 2));

        self.triangle_uvs.a = vec![0.0, aux, aux1]
            .try_into()
            .expect("Expected exactly 3 elements");
        self.triangle_uvs.b = vec![aux, aux1, 0.0]
            .try_into()
            .expect("Expected exactly 3 elements");
        self.triangle_uvs.c = vec![aux1, 0.0, aux]
            .try_into()
            .expect("Expected exactly 3 elements");
    }

    // to 90 degrees right triangle
    fn triangle_arc_lengths(&mut self)  {
        let uvs = &self.triangle_uvs;
        let dot_ab = 0.0;
        self.triangle_arcs.ab = f64::acos(dot_ab);
        self.triangle_arcs.bc =
            f64::acos(uvs.b[0] * uvs.c[0] + uvs.b[1] * uvs.c[1] + uvs.b[2] * uvs.c[2]);
        self.triangle_arcs.ac =
            f64::acos(uvs.a[0] * uvs.c[0] + uvs.a[1] * uvs.c[1] + uvs.a[2] * uvs.c[2]);

        let uvp0 = uvs.a[0] + uvs.b[0] + uvs.c[0];
        let uvp1 = uvs.a[1] + uvs.b[1] + uvs.c[1];
        let uvp2 = uvs.a[2] + uvs.b[2] + uvs.c[2];
        let norm_uvp = f64::sqrt(pow(uvp0, 2) + pow(uvp1, 2) + pow(uvp2, 2));
        let uvp = vec![uvp0 / norm_uvp, uvp1 / norm_uvp, uvp2 / norm_uvp];

        self.triangle_arcs.ap =
            f64::acos(uvs.a[0] * uvp[0] + uvs.a[1] * uvp[1] + uvs.a[2] * uvp[2]);
        self.triangle_arcs.bp =
            f64::acos(uvs.b[0] * uvp[0] + uvs.b[1] * uvp[1] + uvs.b[2] * uvp[2]);
        self.triangle_arcs.cp =
            f64::acos(uvs.c[0] * uvp[0] + uvs.c[1] * uvp[1] + uvs.c[2] * uvp[2]);
    }
}

