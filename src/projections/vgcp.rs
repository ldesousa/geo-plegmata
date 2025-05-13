use std::f64::consts::{E, PI};

use crate::{
    models::common::{Position2D, PositionGeo},
    traits::{
        polyhedron::{ArcLengths, Polyhedron},
        projection::Projection,
    },
    utils::math::{cos, pow, sin, tan, to_rad},
};

use super::constants::COEF_GEOD_TO_AUTH_LAT;

// use super::vgcp::Vgcp;

/// Implementation for Icosahedron Vertex Great Circle Projection.
/// vgcp - Vertex Great Circle Projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub struct Vgcp;

// - conseguir os vertices dos triangulos, achar a configuração do Icosahedron
// - Achar onde os pontos se encontram no triangulo
// https://chatgpt.com/c/68091cde-4428-8002-beb3-6713a35494ec

impl Projection for Vgcp {
    fn forward(&self, positions: Vec<PositionGeo>, shape: &dyn Polyhedron) -> Vec<Position2D> {
        let mut out: Vec<Position2D> = vec![];

        // convert from geodetic to authalic
        let coef_fourier_geod_to_auth = Self::compute_fourier_coefficients(COEF_GEOD_TO_AUTH_LAT);

        // get 3d unit vectors of the icosahedron
        let ico_vectors = shape.get_unit_vectors();
        println!("{:?}", ico_vectors);

        // /// @TODO: ADD TO CONSTANTS OF ICOSAHEDRON
        /// ABC
        let angle_beta: f64 = to_rad(36.0);
        /// BCA
        let angle_gamma: f64 = to_rad(60.0);
        /// BAC
        let angle_alpha: f64 = PI / 2.0;

        let v2d = shape.get_planar_vertexes();

        for position in positions {
            let lon = position.lon;
            let lat = Self::lat_geodetic_to_authalic(position.lat, &coef_fourier_geod_to_auth);
            // Calculate 3d unit vectors for point P
            let vector_3d = Self::to_3d(to_rad(lat), to_rad(lon));

            /// Polyhedron faces
            let faces_length = shape.get_faces();
        //     for index in 0..faces_length {
        //         let face = usize::from(index);
        //         println!("{:?}", index);

        //         /// get 3vector
        //         /// ...
        //         // for p in positions {
        //         // let p = &positions[0];
                let ArcLengths {
                    ab,
                    bp,
                    ap,
                    bc,
                    ac,
                    cp,
                } = shape.get_triangle_arc_lengths(vector_3d);
        //         // println!("{:?}", ac);
        //         // icoVertices -> vector 3d
        //         // vertices5x6 -> 2d vertezes

        //         // let uvs = shape.get_triangle_unit_vectors();

        //         // angle ρ
        //         let rho: f64 = f64::acos(cos(ap) - cos(ab) * cos(bp)) / (sin(ab) * sin(bp));

        //         // /// 1. Calculate delta (δ)
        //         let delta = f64::acos(f64::sin(rho) * f64::cos(ab));

        //         // /// 2. Calculate u
        //         let uv = (angle_beta + angle_gamma - rho - delta)
        //             / (angle_beta + angle_gamma - PI / 2.0);

        //         let cosXpY;
        //         if rho <= pow(E, -9) {
        //             cosXpY = cos(ab);
        //         } else {
        //             cosXpY = 1.0 / (tan(rho) * tan(delta))
        //         }

        //         let xy = f64::sqrt((1.0 - cos(bp)) / (1.0 - cosXpY));

        //         // triangle vertexes
        //         let vx0 = f64::from(v2d[face][0].0);
        //         let vy0 = f64::from(v2d[face][0].1);
        //         let vx1 = f64::from(v2d[face][1].0);
        //         let vy1 = f64::from(v2d[face][1].1);
        //         let vx2 = f64::from(v2d[face][2].0);
        //         let vy2 = f64::from(v2d[face][2].1);
        //         // entre o A e o C que dá o ponto D
        //         let pdi_x = vx2 + (vx0 - vx2) * uv;
        //         let pdi_y = vy2 + (vy0 - vy2) * uv;

        //         // entre o D e o B que dá o ponto P
        //         let pdi_x = pdi_x + (pdi_x - vx1) * xy;
        //         let pdi_y = pdi_y + (pdi_y - vy1) * xy;

        //         out.push(Position2D { x: pdi_x, y: pdi_y });
        //     }
        }

        // // }
        // out
        vec![Position2D { x: 0.0, y: 0.0 }]
    }
    fn inverse(&self) -> String {
        "todo!()".to_string()
    }
}

// impl Projection<Tetra> for Ivgcp {
//     fn forward(positions: Vec<Position>, shape: &Tetra) -> [f64; 2] {
//         todo!()
//     }

//     fn inverse(&self) -> String {
//         todo!()
//     }
// }
