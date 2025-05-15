use std::f64::consts::{E, PI};

use crate::{
    models::{
        common::{Position2D, PositionGeo},
        vector_3d::Vector3D,
    },
    traits::{
        layout::Layout,
        polyhedron::{ArcLengths, Polyhedron},
        projection::Projection,
    },
    utils::math::{cos, pow, sin, tan, to_rad},
};

use super::constants::COEF_GEOD_TO_AUTH_LAT;

// use super::vgcp::Vgcp;

/// Implementation for Vertex Great Circle projection (or van Leeuwen Great Circle projection).
/// vgc - Vertex Great Circle projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub struct Vgc;

impl Projection for Vgc {
    fn forward(
        &self,
        positions: Vec<PositionGeo>,
        polyhedron: Option<&dyn Polyhedron>,
        layout: &dyn Layout,
    ) -> Vec<Position2D> {
        let mut out: Vec<Position2D> = vec![];
        let polyhedron = polyhedron.unwrap();

        // Need the coeficcients to convert from geodetic to authalic
        let coef_fourier_geod_to_auth = Self::compute_fourier_coefficients(COEF_GEOD_TO_AUTH_LAT);

        // get 3d unit vectors of the icosahedron
        let ico_vectors = polyhedron.unit_vectors();
        let triangles_ids = polyhedron.indices();

        // ABC
        let angle_beta: f64 = to_rad(36.0);
        // BCA
        let angle_gamma: f64 = to_rad(60.0);
        // // BAC
        // let angle_alpha: f64 = PI / 2.0;

        let v2d = layout.vertices();

        for position in positions {
            let lon = position.lon;
            let lat = Self::lat_geodetic_to_authalic(position.lat, &coef_fourier_geod_to_auth);
            // Calculate 3d unit vectors for point P
            let vector_3d = Vector3D::from_array(Self::to_3d(to_rad(lat), to_rad(lon)));

            // starting from here you need:
            // - the 3d point that you want to project
            // - the 3d vertexes of the icosahedron
            // - the 2d vertexes of the config 5x6
            // Polyhedron faces
            let faces_length = polyhedron.faces();
            for index in 0..faces_length {
                let face = usize::from(index);
                let ids = triangles_ids[face];

                let triangle_3d = vec![
                    ico_vectors[ids[0] as usize],
                    ico_vectors[ids[1] as usize],
                    ico_vectors[ids[2] as usize],
                ];
                if polyhedron.is_point_in_triangle(vector_3d, triangle_3d.clone()) {
                    let (triangle_3d, triangle_2d) =
                        polyhedron.triangles(layout, vector_3d, triangle_3d, v2d[face]);
                    let ArcLengths { ab, bp, ap, .. } =
                        polyhedron.triangle_arc_lengths(triangle_3d, vector_3d);

                    // angle ρ
                    let rho: f64 = f64::acos(cos(ap) - cos(ab) * cos(bp)) / (sin(ab) * sin(bp));

                    // /// 1. Calculate delta (δ)
                    let delta = f64::acos(f64::sin(rho) * f64::cos(ab));

                    // /// 2. Calculate u
                    let uv = (angle_beta + angle_gamma - rho - delta)
                        / (angle_beta + angle_gamma - PI / 2.0);

                    let cos_xp_y;
                    if rho <= pow(E, -9) {
                        cos_xp_y = cos(ab);
                    } else {
                        cos_xp_y = 1.0 / (tan(rho) * tan(delta))
                    }

                    let xy = f64::sqrt((1.0 - cos(bp)) / (1.0 - cos_xp_y));

                    // triangle vertexes
                    let (p0, p1, p2) = (&triangle_2d[0], &triangle_2d[1], &triangle_2d[2]);

                    // entre o A e o C que dá o ponto D
                    let px = p2.x + (p0.x - p2.x) * uv;
                    let py = p2.y + (p0.y - p2.y) * uv;

                    // entre o D e o B que dá o ponto P
                    let px = px + (px - p1.x) * xy;
                    let py = px + (py - p1.y) * xy;

                    out.push(Position2D { x: px, y: py });
                }
            }
        }

        // // }
        out
        // vec![Position2D { x: 0.0, y: 0.0 }]
    }
    fn inverse(&self) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{layout::rhombic5x6::Rhombic5x6, models::common::PositionGeo, polyhedron::icosahedron::Icosahedron, traits::projection::Projection};

    use super::Vgc;

    fn project_forward() {
        let position = PositionGeo {
            lat: 38.695125,
            lon: -9.222154,
        };
        let projection = Vgc;

        let result = projection.forward(vec![position], Some(&Icosahedron {}), &Rhombic5x6 {});

    }
}
