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
    projections::polyhedron::geometry::Face,
};

use super::polyhedron::Polyhedron;

/// Factory function to create an icosahedron with optimal orientation for DGGS
/// - Almost no vertices on land, which reduces distortion for land-based DGGS queries
/// by avoiding vertex-based singularities over populated areas.
/// - Two vertices on the poles, which ensures better symmetry for polar areas and
/// simplifies some projections.
/// - Rotated implementation optimized for equal-area projections.
/// That means this icosahedron is not a standard implementation but a rotated implementation to fit equal-area projections.
/// The other vertices are on northern and southern hemisphere in two equatorial rings, with alternating longitude.

pub fn new() -> Polyhedron {
    let vertices = create_vertices();
    let faces = create_faces();
    let num_edges = 30; // Icosahedron has 30 edges

    Polyhedron::new(vertices, faces, num_edges)
}

/// Create the 12 icosahedron vertices
fn create_vertices() -> Vec<Vector3D> {
    let mut vertices = Vec::with_capacity(12);
    let z = 1.0 / 5.0_f64.sqrt();
    let r = (1.0 - z.powi(2)).sqrt();

    // North Pole (Vertex 0)
    vertices.push(Vector3D {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    });

    // Upper ring (Vertices 1-5)
    for i in 0..5 {
        let angle = 2.0 * PI * (i as f64) / 5.0;
        vertices.push(Vector3D {
            x: r * angle.cos(),
            y: r * angle.sin(),
            z: z,
        });
    }

    // Lower ring (Vertices 6-10, rotated by 36°)
    for i in 0..5 {
        let angle = 2.0 * PI * (i as f64) / 5.0 + PI / 5.0;
        vertices.push(Vector3D {
            x: r * angle.cos(),
            y: r * angle.sin(),
            z: -z,
        })
    }

    // South Pole (Vertex 11)
    vertices.push(Vector3D {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    });

    vertices
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

    #[test]
    fn test_icosahedron_creation() {
        let ico = new();
        assert_eq!(ico.num_vertices(), 12);
        assert_eq!(ico.num_faces(), 20);
        assert_eq!(ico.num_edges(), 30);
    }
    #[test]
    fn test_face_centers_on_unit_sphere() {
        let ico = new();

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
        let ico = new();

        for i in 0..ico.num_faces() {
            let center = ico.face_center(i);
            assert!(
                ico.is_point_in_face(center, i),
                "Face center not inside face {}",
                i
            );
        }
    }
}
