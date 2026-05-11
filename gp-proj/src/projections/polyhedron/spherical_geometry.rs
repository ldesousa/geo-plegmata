// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
// Modified by Sunayana Ghosh (sunayanag@gmail.com)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms
//! Geometric utilities for polyhedron operations
//!
//! This module provides computational geometry functions for operations on
//! polyhedra in 3D space, commonly used in DGGS applications.
//!
//! # Overview
//!
//! The functions include both planar geometry operations (for triangle containment)
//! and spherical geometry operations (for angle calculations and area computations).
//! They use numerically stable algorithms to handle edge cases and floating-point
//! precision issues.
//!
//! # Usage
//!
//! ```
//! use gp_proj::projections::polyhedron::spherical_geometry;
//! use gp_proj::models::vector_3d::Vector3D;
//!
//! let vertex_a = Vector3D::new(1.0, 0.0, 0.0);
//! let vertex_b = Vector3D::new(0.0, 1.0, 0.0);
//! let vertex_c = Vector3D::new(0.0, 0.0, 1.0);
//! let triangle = [vertex_a, vertex_b, vertex_c];
//! let test_point = Vector3D::new(0.3, 0.3, 0.3).normalize();
//! let is_inside = spherical_geometry::point_in_planar_triangle(test_point, triangle);
//!
//! let vec_a = Vector3D::new(1.0, 0.0, 0.0);
//! let vec_b = Vector3D::new(0.0, 1.0, 0.0);
//! let angle = spherical_geometry::stable_angle_between(vec_a, vec_b);
//! ```

use crate::models::vector_3d::Vector3D;

/// Numerical tolerance for geometric calculations
///
/// This tolerance accounts for floating-point precision errors in
/// barycentric coordinate calculations and angle computations.
///
/// The value is chosen to be small enough to maintain precision while
/// being large enough to handle typical floating-point rounding errors.
pub const GEOMETRIC_TOLERANCE: f64 = 1e-10;

/// Degenerate triangle detection threshold
///
/// Triangles with determinants smaller than this threshold are considered
/// degenerate and will cause point-in-triangle tests to return false.
pub const DEGENERATE_TRIANGLE_THRESHOLD: f64 = 1e-10;

/// Test if a point lies inside a planar triangle using barycentric coordinates
///
/// This function uses standard Cartesian barycentric coordinate calculation in 3D space.
/// It treats the triangle vertices as 3D Cartesian points, not as points on a sphere.
/// This is a planar geometric test, not true spherical geometry.
///
/// # Arguments
/// * `point` - Point to test (3D Cartesian coordinates)
/// * `triangle` - Three vertices of triangle (3D Cartesian coordinates)
///
/// # Returns
/// `true` if point is inside triangle (including boundaries with tolerance), `false` otherwise
///
/// # Algorithm
///
/// Uses standard planar barycentric coordinates computed via dot products of Cartesian edge vectors.
/// A point is inside the triangle if all barycentric coordinates are non-negative
/// (with tolerance for numerical stability).
///
/// # Examples
/// ```
/// use gp_proj::models::vector_3d::Vector3D;
/// use gp_proj::projections::polyhedron::spherical_geometry::point_in_planar_triangle;
///
/// let vertex_a = Vector3D::new(1.0, 0.0, 0.0);
/// let vertex_b = Vector3D::new(0.0, 1.0, 0.0);
/// let vertex_c = Vector3D::new(0.0, 0.0, 1.0);
/// let triangle = [vertex_a, vertex_b, vertex_c];
/// let test_point = Vector3D::new(0.3, 0.3, 0.3).normalize();
/// let is_inside = point_in_planar_triangle(test_point, triangle);
/// assert!(is_inside);
/// ```
///
/// # Performance
/// O(1) - constant time computation with ~20 floating-point operations
pub fn point_in_planar_triangle(point: Vector3D, triangle: [Vector3D; 3]) -> bool {
    let [v0, v1, v2] = triangle;

    // Convert to barycentric coordinates using Cartesian edge vectors
    let v0v1 = v1 - v0;
    let v0v2 = v2 - v0;
    let v0p = point - v0;

    // Compute dot products for barycentric coordinate calculation
    let dot00 = v0v2.dot(v0v2);
    let dot01 = v0v2.dot(v0v1);
    let dot02 = v0v2.dot(v0p);
    let dot11 = v0v1.dot(v0v1);
    let dot12 = v0v1.dot(v0p);

    // Compute barycentric coordinates
    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < DEGENERATE_TRIANGLE_THRESHOLD {
        return false; // Degenerate triangle
    }

    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    // Point is in triangle if all barycentric coordinates are non-negative
    // Use tolerance to handle floating-point precision near boundaries
    u >= -GEOMETRIC_TOLERANCE && v >= -GEOMETRIC_TOLERANCE && (u + v) <= 1.0 + GEOMETRIC_TOLERANCE
}

/// Test if a point lies inside a spherical triangle using barycentric coordinates
///
/// This function uses standard Spherical barycentric coordinate calculation in 3D space.
/// It treats the triangle vertices as points on a sphere.
///
/// # Arguments
/// * `point` - Point to test (3D Spherical coordinates)
/// * `triangle` - Three vertices of triangle (3D Spherical coordinates)
///
/// # Returns
/// `true` if point is inside triangle, `false` otherwise
///
/// # Algorithm
///
/// Computes the spherical barycentric areas.
/// A point is inside the triangle if all barycentric weights are non-negative or the it's sum is 1.
///
/// # Examples
/// ```
pub fn point_in_spherical_triangle(p: Vector3D, triangle: [Vector3D; 3]) -> bool {
    let [a, b, c] = triangle;

    let n_ab = a.cross(b);
    let n_bc = b.cross(c);
    let n_ca = c.cross(a);

    // Check if P is on same side as opposite vertex
    let inside_ab = n_ab.dot(c) * n_ab.dot(p) >= -DEGENERATE_TRIANGLE_THRESHOLD;
    let inside_bc = n_bc.dot(a) * n_bc.dot(p) >= -DEGENERATE_TRIANGLE_THRESHOLD;
    let inside_ca = n_ca.dot(b) * n_ca.dot(p) >= -DEGENERATE_TRIANGLE_THRESHOLD;

    inside_ab && inside_bc && inside_ca
}


/// Compute angle between two unit vectors using numerically stable method
///
/// This function uses the atan2 method with cross product magnitude for better
/// numerical stability than the traditional acos(dot_product) approach, especially
/// for small angles and vectors that are nearly parallel or antiparallel.
///
/// # Arguments  
/// * `u`, `v` - Unit vectors on the sphere
///
/// # Returns
/// Angle in radians between the vectors (0 to π)
///
/// # Algorithm
///
/// Uses the relationship: angle = atan2(|u × v|, u · v)
/// This is more stable than acos(u · v) because:
/// - atan2 handles all quadrants correctly
/// - Cross product magnitude provides better precision for small angles
/// - Avoids domain errors near ±1 that can occur with acos
///
/// # Examples
/// ```
/// use gp_proj::models::vector_3d::Vector3D;
/// use gp_proj::projections::polyhedron::spherical_geometry::stable_angle_between;
/// use std::f64::consts::PI;
///
/// let vec_a = Vector3D::new(1.0, 0.0, 0.0);
/// let vec_b = Vector3D::new(0.0, 1.0, 0.0);
/// let angle = stable_angle_between(vec_a, vec_b);
/// let expected_angle = PI / 2.0; // 90 degrees
/// assert!((angle - expected_angle).abs() < 1e-10);
/// ```
///
/// # Performance
/// O(1) - constant time with ~15 floating-point operations
pub fn stable_angle_between(u: Vector3D, v: Vector3D) -> f64 {
    let cross = u.cross(v);
    let cross_magnitude = cross.length();
    let dot = u.dot(v);

    // atan2 handles all quadrants correctly and is more stable than acos
    cross_magnitude.atan2(dot)
}

/// Compute barycentric coordinates for a point relative to a spherical triangle
///
/// Returns the barycentric coordinates (u, v, w) where w = 1 - u - v.
/// These coordinates can be used for interpolation or to determine if a point
/// is inside the triangle (all coordinates ≥ 0).
///
/// # Arguments
/// * `point` - Point to compute coordinates for (unit vector)
/// * `triangle` - Three vertices of the spherical triangle (unit vectors)
///
/// # Returns  
/// `Some((u, v, w))` with barycentric coordinates, or `None` if triangle is degenerate
///
/// # Interpretation
/// - u, v, w represent the "weights" of each triangle vertex
/// - u corresponds to triangle[0], v to triangle[1], w to triangle[2]  
/// - u + v + w = 1.0 (within floating-point precision)
/// - Point is inside triangle if u ≥ 0, v ≥ 0, w ≥ 0
///
/// # Examples
/// ```
/// use gp_proj::models::vector_3d::Vector3D;
/// use gp_proj::projections::polyhedron::spherical_geometry::barycentric_coordinates;
///
/// let vertex_a = Vector3D::new(1.0, 0.0, 0.0);
/// let vertex_b = Vector3D::new(0.0, 1.0, 0.0);
/// let vertex_c = Vector3D::new(0.0, 0.0, 1.0);
/// let triangle = [vertex_a, vertex_b, vertex_c];
/// let point = Vector3D::new(0.3, 0.3, 0.3).normalize();
///
/// if let Some((u, v, w)) = barycentric_coordinates(point, triangle) {
///     let interpolated = u * triangle[0] + v * triangle[1] + w * triangle[2];
///     // interpolated ≈ point (for points inside triangle)
///     assert!((u + v + w - 1.0).abs() < 1e-10); // coordinates sum to 1
/// }
/// ```
/// 
// @TODO - the barycentric_coordinates function is not for the sphere, 
// the following function seems to be the correct, needs changing
// pub fn compute_spherical_barycentric(
//     point: Vector3D,
//     v0: Vector3D,
//     v1: Vector3D,
//     v2: Vector3D,
// ) -> (f64, f64, f64) {
//     let total_area = spherical_triangle_area([v0, v1, v2]).unwrap();
//     let area0 = spherical_triangle_area([point, v1, v2]).unwrap();
//     let area1 = spherical_triangle_area([v0, point, v2]).unwrap();
//     let area2 = spherical_triangle_area([v0, v1, point]).unwrap();

//     (area0 / total_area, area1 / total_area, area2 / total_area)
// }

pub fn barycentric_coordinates(
    point: Vector3D,
    triangle: [Vector3D; 3],
) -> Option<(f64, f64, f64)> {
    let [v0, v1, v2] = triangle;

    // Check if point is very close to any vertex (handle special case)
    let vertex_tolerance = 1e-12;
    if (point - v0).length() < vertex_tolerance {
        return Some((1.0, 0.0, 0.0));
    }
    if (point - v1).length() < vertex_tolerance {
        return Some((0.0, 1.0, 0.0));
    }
    if (point - v2).length() < vertex_tolerance {
        return Some((0.0, 0.0, 1.0));
    }

    let v0v1 = v1 - v0;
    let v0v2 = v2 - v0;
    let v0p = point - v0;

    let dot00 = v0v2.dot(v0v2);
    let dot01 = v0v2.dot(v0v1);
    let dot02 = v0v2.dot(v0p);
    let dot11 = v0v1.dot(v0v1);
    let dot12 = v0v1.dot(v0p);

    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < DEGENERATE_TRIANGLE_THRESHOLD {
        return None; // Degenerate triangle
    }

    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    let w = 1.0 - u - v;

    Some((u, v, w))
}

/// Compute the spherical area of a triangle using Girard's theorem
///
/// Calculates the area of a spherical triangle on the unit sphere using
/// the spherical excess formula: Area = E where E = (A + B + C) - π
/// and A, B, C are the interior angles of the spherical triangle.
///
/// # Arguments
/// * `triangle` - Three vertices of the spherical triangle (unit vectors)
///
/// # Returns
/// Area of the spherical triangle in steradians, or None if triangle is degenerate
///
/// # Examples
/// ```
/// use gp_proj::models::vector_3d::Vector3D;
/// use gp_proj::projections::polyhedron::spherical_geometry::spherical_triangle_area;
/// use std::f64::consts::PI;
///
/// // Create a triangle that covers 1/8 of the sphere (octant)
/// let vertex_a = Vector3D::new(1.0, 0.0, 0.0);
/// let vertex_b = Vector3D::new(0.0, 1.0, 0.0);
/// let vertex_c = Vector3D::new(0.0, 0.0, 1.0);
/// let triangle = [vertex_a, vertex_b, vertex_c];
/// let area = spherical_triangle_area(triangle).unwrap_or(0.0);
///
/// // Area should be π/2 (1/8 of sphere surface area 4π)
/// let expected_area = PI / 2.0;
/// assert!((area - expected_area).abs() < 1e-10);
/// ```
pub fn spherical_triangle_area(triangle: [Vector3D; 3]) -> Option<f64> {
    let [a, b, c] = triangle;

    // Use the vector triple product formula for spherical triangle area
    // Area = 2 * atan2(|a·(b×c)|, 1 + a·b + b·c + c·a)
    let cross_bc = b.cross(c);
    let triple_product = a.dot(cross_bc).abs();

    let dot_ab = a.dot(b);
    let dot_bc = b.dot(c);
    let dot_ca = c.dot(a);

    let denominator = 1.0 + dot_ab + dot_bc + dot_ca;

    if denominator.abs() < DEGENERATE_TRIANGLE_THRESHOLD {
        return None; // Degenerate triangle
    }

    Some(2.0 * triple_product.atan2(denominator))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    #[test]
    fn test_point_in_planar_triangle_center() {
        // Test that triangle center is inside triangle
        let triangle = [
            Vector3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ];

        let center = (triangle[0] + triangle[1] + triangle[2]).normalize();
        assert!(point_in_planar_triangle(center, triangle));
    }

    #[test]
    fn test_point_in_planar_triangle_outside() {
        let triangle = [
            Vector3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ];

        // Point clearly outside the triangle
        let outside_point = Vector3D {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        };
        assert!(!point_in_planar_triangle(outside_point, triangle));
    }

    #[test]
    fn test_point_in_planar_triangle_vertex() {
        let triangle = [
            Vector3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ];

        // Vertices should be considered inside (on boundary)
        assert!(point_in_planar_triangle(triangle[0], triangle));
        assert!(point_in_planar_triangle(triangle[1], triangle));
        assert!(point_in_planar_triangle(triangle[2], triangle));
    }

    #[test]
    fn test_stable_angle_between_orthogonal() {
        let u = Vector3D {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let v = Vector3D {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };

        let angle = stable_angle_between(u, v);
        assert!((angle - FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn test_stable_angle_between_parallel() {
        let u = Vector3D {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let v = Vector3D {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };

        let angle = stable_angle_between(u, v);
        assert!(angle.abs() < 1e-10);
    }

    #[test]
    fn test_stable_angle_between_antiparallel() {
        let u = Vector3D {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let v = Vector3D {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        };

        let angle = stable_angle_between(u, v);
        assert!((angle - PI).abs() < 1e-10);
    }

    #[test]
    fn test_barycentric_coordinates_center() {
        let triangle = [
            Vector3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ];

        let center = (triangle[0] + triangle[1] + triangle[2]).normalize();
        if let Some((u, v, w)) = barycentric_coordinates(center, triangle) {
            // All coordinates should be positive for center point
            assert!(u > 0.0);
            assert!(v > 0.0);
            assert!(w > 0.0);
            // Should sum to 1.0
            assert!((u + v + w - 1.0).abs() < 1e-10);
        } else {
            panic!("Barycentric coordinates should exist for valid triangle");
        }
    }

    #[test]
    fn test_barycentric_coordinates_vertex() {
        let triangle = [
            Vector3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ];

        // Test vertex coordinates - use looser tolerance for vertex cases
        if let Some((u, v, w)) = barycentric_coordinates(triangle[0], triangle) {
            assert!((u - 1.0).abs() < 1e-6, "u = {}, expected ≈ 1.0", u);
            assert!(v.abs() < 1e-6, "v = {}, expected ≈ 0.0", v);
            assert!(w.abs() < 1e-6, "w = {}, expected ≈ 0.0", w);
        }
    }

    #[test]
    fn test_spherical_triangle_area_octant() {
        // Triangle covering 1/8 of sphere (π/2 steradians)
        let triangle = [
            Vector3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ];

        if let Some(area) = spherical_triangle_area(triangle) {
            // Should be π/2 steradians for this triangle
            assert!(
                (area - FRAC_PI_2).abs() < 1e-2,
                "Expected π/2, got {}",
                area
            );
        } else {
            panic!("Area calculation should succeed for valid triangle");
        }
    }

    #[test]
    fn test_constants_are_reasonable() {
        // Tolerance should be small but not too small
        assert!(GEOMETRIC_TOLERANCE > 0.0);
        assert!(GEOMETRIC_TOLERANCE < 1e-5);

        assert!(DEGENERATE_TRIANGLE_THRESHOLD > 0.0);
        assert!(DEGENERATE_TRIANGLE_THRESHOLD < 1e-5);
    }
}
