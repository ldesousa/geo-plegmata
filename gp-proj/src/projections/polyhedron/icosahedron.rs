// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
// Modified by Sunayana Ghosh (sunayanag@gmail.com)
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use std::f64::consts::PI;

use crate::{
    models::vector_3d::Vector3D,
    projections::polyhedron::geometry::{Face, Orientation},
};

use super::polyhedron::Polyhedron;

/// Factory function to create an icosahedron with the given orientation.
///
/// `orientation` specifies where vertex 0 (the top vertex, tip of the five top
/// triangles) is placed on the globe. Use Orientation::DGGS_OPTIMAL for the 
/// standard land-avoiding placement (with one vertex positioned on land) or Orientation::POLAR for the
/// canonical mathematical alignment with a vertex at each pole.
pub fn new(orientation: Orientation) -> Polyhedron {
    let vertices = create_vertices(orientation);
    let faces = create_faces();
    Polyhedron::new(vertices, faces, 30)
}

/// Build the 12 vertices starting from the canonical polar alignment, then rotate
/// the whole icosahedron so vertex 0 lands at the requested orientation.
///
/// The rotation is decomposed into:
///   1. pitch by the colatitude (PI/2 − lat): moves vertex 0 from (0,0,1) to
///      (cos(lat), 0, sin(lat)) — no longitude change yet.
///   2. yaw by the longitude: sweeps that point to the final (lat, lon) position.
///
/// Applying both steps to every vertex preserves topology and all inter-vertex
/// angular distances.
fn create_vertices(orientation: Orientation) -> Vec<Vector3D> {
    let z = 1.0 / 5.0_f64.sqrt();
    let r = (1.0 - z.powi(2)).sqrt();

    let mut vertices = Vec::with_capacity(12);

    // Canonical orientation: vertex 0 at north pole
    vertices.push(Vector3D { x: 0.0, y: 0.0, z: 1.0 });

    // Upper ring (Vertices 1-5)
    for i in 0..5 {
        let angle = 2.0 * PI * (i as f64) / 5.0;
        vertices.push(Vector3D { x: r * angle.cos(), y: r * angle.sin(), z });
    }

    // Lower ring (Vertices 6-10, rotated by 36° relative to upper ring)
    for i in 0..5 {
        let angle = 2.0 * PI * (i as f64) / 5.0 + PI / 5.0;
        vertices.push(Vector3D { x: r * angle.cos(), y: r * angle.sin(), z: -z });
    }

    // South pole (Vertex 11)
    vertices.push(Vector3D { x: 0.0, y: 0.0, z: -1.0 });

    let lat = orientation.lat_deg.to_radians();
    let lon = orientation.lon_deg.to_radians();
    let colatitude = PI / 2.0 - lat;

    vertices.iter().map(|&v| v.pitch(colatitude).yaw(lon)).collect()
}

/// Create the 20 triangular faces of the icosahedron
fn create_faces() -> Vec<Face> {
    // A => 0, B => 1, C => 2, D => 3, E => 4, F => 5,
    // G => 6, H => 7, I => 8, J => 9, K => 10, L => 11
    vec![
        Face::Triangle([2, 1, 0]),   // C, B, A
        Face::Triangle([2, 1, 6]),   // C, B, G
        Face::Triangle([3, 2, 0]),   // D, C, A
        Face::Triangle([3, 2, 7]),   // D, C, H
        Face::Triangle([4, 3, 0]),   // E, D, A
        Face::Triangle([4, 3, 8]),   // E, D, I
        Face::Triangle([5, 4, 0]),   // F, E, A
        Face::Triangle([5, 4, 9]),   // F, E, J
        Face::Triangle([1, 5, 0]),   // B, F, A
        Face::Triangle([1, 5, 10]),  // B, F, K
        Face::Triangle([7, 6, 2]),   // H, G, C
        Face::Triangle([7, 6, 11]),  // H, G, L
        Face::Triangle([8, 7, 3]),   // I, H, D
        Face::Triangle([8, 7, 11]),  // I, H, L
        Face::Triangle([9, 8, 4]),   // J, I, E
        Face::Triangle([9, 8, 11]),  // J, I, L
        Face::Triangle([10, 9, 5]),  // K, J, F
        Face::Triangle([10, 9, 11]), // K, J, L
        Face::Triangle([6, 10, 1]),  // G, K, B
        Face::Triangle([6, 10, 11]), // G, K, L
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dggs() -> Polyhedron {
        new(Orientation::DGGS_OPTIMAL)
    }

    #[test]
    fn test_icosahedron_creation() {
        let ico = dggs();
        assert_eq!(ico.num_vertices(), 12);
        assert_eq!(ico.num_faces(), 20);
        assert_eq!(ico.num_edges(), 30);
    }

    #[test]
    fn test_face_centers_on_unit_sphere() {
        let ico = dggs();
        for i in 0..ico.num_faces() {
            let center = ico.face_center(i);
            let norm = center.dot(center);
            assert!(
                (norm - 1.0).abs() < 1e-5,
                "Face center {} not normalized",
                i
            );
        }
    }

    #[test]
    fn test_face_centers_inside_faces() {
        let ico = dggs();
        for i in 0..ico.num_faces() {
            let center = ico.face_center(i);
            assert!(
                ico.is_point_in_face(center, i),
                "Face center not inside face {}",
                i
            );
        }
    }

    #[test]
    fn test_polar_orientation_vertex_at_north_pole() {
        let ico = new(Orientation::POLAR);
        let v0 = ico.vertices()[0];
        assert!((v0.x).abs() < 1e-10);
        assert!((v0.y).abs() < 1e-10);
        assert!((v0.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_dggs_optimal_vertex_position() {
        let ico = new(Orientation::DGGS_OPTIMAL);
        let v0 = ico.vertices()[0];
        let lat = 58.397145907431_f64.to_radians();
        let lon = 11.20_f64.to_radians();
        let expected_x = lat.cos() * lon.cos();
        let expected_y = lat.cos() * lon.sin();
        let expected_z = lat.sin();
        assert!((v0.x - expected_x).abs() < 1e-10, "x mismatch");
        assert!((v0.y - expected_y).abs() < 1e-10, "y mismatch");
        assert!((v0.z - expected_z).abs() < 1e-10, "z mismatch");
    }

    #[test]
    fn test_custom_orientation() {
        let orientation = Orientation::new(0.0, 0.0);
        let ico = new(orientation);
        let v0 = ico.vertices()[0];
        // lat=0, lon=0 => vertex 0 on the equator at prime meridian: (1, 0, 0)
        assert!((v0.x - 1.0).abs() < 1e-10, "x mismatch");
        assert!((v0.y).abs() < 1e-10, "y mismatch");
        assert!((v0.z).abs() < 1e-10, "z mismatch");
    }
}
