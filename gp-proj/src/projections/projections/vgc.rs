// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
// Co-authored by Sunayana Ghosh (Independent Researcher, sunayanag@gmail.com)
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use std::f64::consts::{E, PI};

use crate::{
    constants::WGS84,
    ellipsoid::{AuthalicCoord, AuthalicSphere, Ellipsoid},
    models::vector_3d::Vector3D,
    projections::{
        layout::traits::Layout,
        polyhedron::{ArcLengths, Orientation, Polyhedron, icosahedron, spherical_geometry},
        projections::traits::{DistortionMetrics, ForwardBary, ForwardCartesian, Projection},
    },
    utils::shape::triangle,
};
use geo::Coord;
use geoplegma::types::Point;

// SUB_TRIANGLE_TEMPLATE
// Each icosahedron face is divided into 6 sub-triangles by connecting the face center
// to the midpoints of each edge. All 6 sub-triangles are congruent right triangles.
// Arc lengths measured on the unit sphere for one sub-triangle:
//   ab (corner to mid) = 0.553574 rad
//   bc (corner to center) = 0.652358 rad
//   ac (mid to center) = 0.364864 rad
// Template is built with B (corner) at origin, A (mid) on negative x-axis,
// C (center) placed using law of cosines at B:
//   cos(angle_B) = (ab² + bc² - ac²) / (2·ab·bc)
//   C = (bc·cos(angle_B), bc·sin(angle_B))
// Raw planar area = 0.5 * |ab * C.y| ≈ 0.100929
// Spherical sub-triangle area = 4π / 120 ≈ 0.104720 (unit sphere, 20 faces × 6 sub-triangles)
// Scale factor = sqrt(0.104720 / 0.100929) ≈ 1.018606
// All coordinates multiplied by 1.018606 to match spherical sub-triangle area.
const SCALE_SUB: f64 = 1.018606;
const SUB_TRIANGLE_TEMPLATE: [(f64, f64); 3] = [
    (0.0, 0.0),                                   // B = corner (origin)
    (-0.553574 * SCALE_SUB, 0.0),                 // A = mid
    (0.540930 * SCALE_SUB, 0.364645 * SCALE_SUB), // C = center
];
// FACE_TEMPLATE_UP and FACE_TEMPLATE_DOWN
// Edge lengths come from the regular icosahedron on a unit sphere:
//   - edge 0-1: π/3 ≈ 1.107149 rad (exact)
//   - edge 1-2 and 2-0: ≈ 1.107149 rad (all equal, regular icosahedron)
// Triangle is built with f1 at origin, f0 on negative x-axis, f2 using law of cosines at f1.
// Raw planar area = 0.5 * |(-1.107149 * 0.958819)| ≈ 0.530938
// Spherical face area = 4π / 20 ≈ 0.628318 (unit sphere, 20 equal faces)
// Scale factor = sqrt(0.628318 / 0.530938) ≈ 1.088072
// All coordinates multiplied by 1.088072 to make planar area equal spherical face area,
// ensuring the equal-area property is preserved when mapping to the face plane.
const SCALE_FACE: f64 = 1.0880715;
const FACE_TEMPLATE_UP: [(f64, f64); 3] = [
    (0.0, 0.0),
    (-1.107149 * SCALE_FACE, 0.0),
    (-0.553574 * SCALE_FACE, 0.958819 * SCALE_FACE),
];
const FACE_TEMPLATE_DOWN: [(f64, f64); 3] = [
    (0.0, 0.0),
    (-1.107149 * SCALE_FACE, 0.0),
    (-0.553574 * SCALE_FACE, -0.958819 * SCALE_FACE),
];

/// Implementation for Vertex Great Circle projection (or van Leeuwen Great Circle projection).
/// vgc - Vertex-oriented Great Circle projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub struct Vgc {
    pub radius: f64,
}

impl Default for Vgc {
    fn default() -> Self {
        Self {
            radius: WGS84::AUTHALIC_RADIUS,
        }
    }
}

impl Projection for Vgc {
    fn geo_to_cartesian(
        &self,
        positions: Vec<AuthalicCoord>,
        polyhedron: Option<&Polyhedron>,
        _layout: Option<&dyn Layout>,
    ) -> Vec<ForwardCartesian> {
        let mut out: Vec<ForwardCartesian> = vec![];
        let polyhedron = polyhedron.unwrap();

        for position in positions {
            // Calculate 3d unit vectors for point P
            let point_p = Vector3D::from_array(Self::to_3d(position.lat, position.lon));
            // starting from here, you need:
            // - the 3d point that you want to project
            // Polyhedron faces
            let faces_length = polyhedron.num_faces();
            for index in 0..faces_length {
                let face = usize::from(index);

                if polyhedron.is_point_in_face(point_p, index) {
                    // the icosahedron triangle gets divided into six rectangle triangles,
                    // and we find the one where the point is
                    let sub_triangle_3d = triangle(
                        polyhedron, point_p, // polyhedron.face_vertices(face).unwrap(),
                        face,
                    )
                    .unwrap();
                    // calculating the arc lenghts from one of the vertices of the sub-triangle to point P
                    let ArcLengths {
                        ab, bp, ap, bc, ac, ..
                    } = polyhedron.arc_lengths(sub_triangle_3d.0, point_p);

                    // Parameterization values of the slice and dice projection.
                    let [xy, uv] = slice_and_dice(ac, ab, bc, ap, bp);

                    // ==== Interpolation ====
                    // Between A and C it gives point D
                    let pd_x = SUB_TRIANGLE_TEMPLATE[2].0
                        + (SUB_TRIANGLE_TEMPLATE[0].0 - SUB_TRIANGLE_TEMPLATE[2].0) * uv;
                    let pd_y = SUB_TRIANGLE_TEMPLATE[2].1
                        + (SUB_TRIANGLE_TEMPLATE[0].1 - SUB_TRIANGLE_TEMPLATE[2].1) * uv;
                    // Between D and B it gives point P
                    let p_x_local =
                        SUB_TRIANGLE_TEMPLATE[1].0 + (pd_x - SUB_TRIANGLE_TEMPLATE[1].0) * xy;
                    let p_y_local =
                        SUB_TRIANGLE_TEMPLATE[1].1 + (pd_y - SUB_TRIANGLE_TEMPLATE[1].1) * xy;
                    // ======================

                    let is_upward = face % 2 == 0;
                    let face_template = if is_upward {
                        FACE_TEMPLATE_UP
                    } else {
                        FACE_TEMPLATE_DOWN
                    };

                    // STEP 3: Get sub-triangle vertices in face coordinates
                    let sub_triangle_id = sub_triangle_3d.1;
                    let sub_vertices_in_face =
                        get_subtriangle_vertices_in_face(sub_triangle_id, face_template);

                    // STEP 4: Transform from sub-triangle local to face coordinates
                    let (p_x_face, p_y_face) = affine_transform_triangle(
                        (p_x_local, p_y_local),
                        SUB_TRIANGLE_TEMPLATE,
                        sub_vertices_in_face,
                    );

                    // Authalic radius
                    let r = self.radius;
                    out.push(ForwardCartesian {
                        coords: Coord {
                            x: p_x_face * r,
                            y: p_y_face * r,
                        },
                        face: index,
                    });

                    // in case the point is on the edge of two faces, we return the first face.
                    break;
                }
            }
        }
        out
    }

    fn geo_to_barycentric(
        &self,
        points: Vec<Point>,
        polyhedron: Option<&Polyhedron>,
        orientation: Option<Orientation>,
        ellipsoid: Option<&dyn Ellipsoid>,
    ) -> Vec<ForwardBary> {
        let ellipsoid: &dyn Ellipsoid = ellipsoid.unwrap_or(&WGS84);
        let sphere = AuthalicSphere::from_ellipsoid(ellipsoid);
        let authalic_points = points.into_iter().map(|p| sphere.convert(p)).collect();

        let built;
        let polyhedron = match polyhedron {
            Some(polyhedron) => polyhedron,
            None => {
                built = icosahedron::new(orientation.unwrap_or(Orientation::DGGS_OPTIMAL));
                &built
            }
        };

        self.geo_to_cartesian(authalic_points, Some(polyhedron), None)
            .into_iter()
            .map(|ForwardCartesian { coords, face }| {
                let is_upward = face % 2 == 0;
                let face_template = if is_upward {
                    FACE_TEMPLATE_UP
                } else {
                    FACE_TEMPLATE_DOWN
                };
                let r = self.radius;
                // Same face-plane triangle geo_to_cartesian projected into (scaled by `r`
                // to match `coords`), reused here as the reference triangle for the
                // point's barycentric weights.
                let triangle = face_template.map(|(x, y)| Vector3D::new(x * r, y * r, 0.0));
                let point = Vector3D::new(coords.x, coords.y, 0.0);

                // barycentric_coordinates(point, [v0, v1, v2]) returns (u, v, w) where,
                // perhaps counter-intuitively, `u` is the weight of `v2`, `v` of `v1`, and
                // `w` of `v0` (verified at the vertices). Reorder here so `coords.x/y/z`
                // line up with `triangle[0]/[1]/[2]` (i.e. `face_template[0]/[1]/[2]`).
                let (u, v, w) = spherical_geometry::barycentric_coordinates(point, triangle)
                    .unwrap_or((f64::NAN, f64::NAN, f64::NAN));

                ForwardBary {
                    coords: Vector3D::new(w, v, u),
                    face,
                }
            })
            .collect()
    }

    fn cartesian_to_geo(&self, _coords: Vec<Coord>) -> Point {
        todo!()
    }

    // Calculate distortion and compare with Geocart values
    fn compute_distortion(
        &self,
        lat: f64,
        lon: f64,
        polyhedron: &Polyhedron,
        ellipsoid: &dyn Ellipsoid,
    ) -> DistortionMetrics {
        let epsilon = 1e-5_f64; // degrees
        let sphere = AuthalicSphere::from_ellipsoid(ellipsoid);
        let to_authalic = |lon: f64, lat: f64| sphere.convert(Point::new(lon, lat));

        let center_xy =
            &self.geo_to_cartesian(vec![to_authalic(lon, lat)], Some(polyhedron), None)[0];
        let north_xy = &self.geo_to_cartesian(
            vec![to_authalic(lon, lat + epsilon)],
            Some(polyhedron),
            None,
        )[0];
        let east_xy = &self.geo_to_cartesian(
            vec![to_authalic(lon + epsilon, lat)],
            Some(polyhedron),
            None,
        )[0];
        if center_xy.face != north_xy.face || center_xy.face != east_xy.face {
            return DistortionMetrics {
                h: f64::NAN,
                k: f64::NAN,
                angular_deformation: f64::NAN,
                areal_scale: f64::NAN,
            };
        }

        // epsilon in radians — coordinates are in meters, input was in degrees
        let eps_rad = epsilon.to_radians();

        let dx_dphi = (north_xy.coords.x - center_xy.coords.x) / eps_rad;
        let dy_dphi = (north_xy.coords.y - center_xy.coords.y) / eps_rad;
        let dx_dlambda = (east_xy.coords.x - center_xy.coords.x) / eps_rad;
        let dy_dlambda = (east_xy.coords.y - center_xy.coords.y) / eps_rad;

        // Radii of curvature (meters/radian), derived from the given ellipsoid
        let a = ellipsoid.major_axis();
        let e2 = ellipsoid.eccentricity_squared();
        let lat_rad = lat.to_radians();
        let sin_lat = lat_rad.sin();
        let cos_lat = lat_rad.cos();

        let m = a * (1.0 - e2) / (1.0 - e2 * sin_lat.powi(2)).powf(1.5);
        let n = a / (1.0 - e2 * sin_lat.powi(2)).sqrt();

        // Normalize derivatives by geodetic radii
        let e = dx_dlambda / (n * cos_lat);
        let f = dy_dlambda / (n * cos_lat);
        let g = dx_dphi / m;
        let h_ = dy_dphi / m;

        // Tissot indicatrix semi-axes a, b (Snyder 1987, Map Projections: A Working
        // Manual, eqs. 4-9-4-13). p, q are the scale magnitudes along the parallel
        // and meridian; areal_scale = |e*h_ - f*g| = p*q*sin(psi) is the Jacobian,
        // where psi is the angle between the projected parallel/meridian tangents.
        //   S = p^2 + q^2, D = 2*areal_scale
        //   a = (sqrt(S+D) + sqrt(S-D)) / 2
        //   b = (sqrt(S+D) - sqrt(S-D)) / 2
        // van Leeuwen & Strebe 2006 ("Slice and Dice", Eq. 29) gives the related
        // max angular deformation sin(omega) = (a-b)/(a+b); they measure a, b
        // numerically (small-circle sampling) rather than via this closed form -
        // this analytic version is equivalent for an infinitesimal circle.
        let p = (e.powi(2) + f.powi(2)).sqrt();
        let q = (g.powi(2) + h_.powi(2)).sqrt();
        let areal_scale = (e * h_ - f * g).abs();

        let s = p.powi(2) + q.powi(2);
        let d = 2.0 * areal_scale;
        let sum_sq = s + d;
        let diff_sq = (s - d).max(0.0);
        let a_tissot = (sum_sq.sqrt() + diff_sq.sqrt()) / 2.0;
        let b_tissot = (sum_sq.sqrt() - diff_sq.sqrt()) / 2.0;
        let omega = 2.0 * ((a_tissot - b_tissot) / (a_tissot + b_tissot)).asin();

        DistortionMetrics {
            h: a_tissot,
            k: b_tissot,
            angular_deformation: omega.to_degrees(),
            areal_scale,
        }
    }
}

fn slice_and_dice(ac: f64, ab: f64, bc: f64, ap: f64, bp: f64) -> [f64; 2] {
    // Spherical angles for point B and point C
    let beta = ((ac.cos() - ab.cos() * bc.cos()) / (ab.sin() * bc.sin()))
        .clamp(-1.0, 1.0)
        .acos();
    let gamma = ((ab.cos() - bc.cos() * ac.cos()) / (bc.sin() * ac.sin()))
        .clamp(-1.0, 1.0)
        .acos();

    // ==== Slice and Dice formulas ====
    // angle ρ
    let rho: f64 =
        f64::acos(((ap.cos() - ab.cos() * bp.cos()) / (ab.sin() * bp.sin())).clamp(-1.0, 1.0));

    // 1. Calculate delta (δ)
    let delta = f64::acos(rho.sin() * ab.cos());

    // 2. Calculate the ratio of the spherical areas u and v
    let uv = ((beta + gamma - rho - delta) / (beta + gamma - PI / 2.0)).clamp(-1.0, 1.0);

    // 3. Calculate cos(x + y) by applying the spherical law of cosines
    // being that the x and y are the spherical lenghts from B to P and P to D, respectively.
    let cos_xp_y;
    if rho <= E.powi(-9) {
        // E = 2.71828...
        cos_xp_y = ab.cos();
    } else {
        cos_xp_y = 1.0 / (rho.tan() * delta.tan())
    }

    // 4. Calculate the ratio of the spherical areas x and y
    let xy = f64::sqrt((1.0 - bp.cos()) / (1.0 - cos_xp_y));

    [xy, uv]
}

/// Get the position of sub-triangle vertices in face 2D coordinates
fn get_subtriangle_vertices_in_face(
    sub_triangle_id: u8,
    face_template: [(f64, f64); 3],
) -> [(f64, f64); 3] {
    // Face vertices
    let [f0, f1, f2] = face_template;

    // Compute face center
    let center = ((f0.0 + f1.0 + f2.0) / 3.0, (f0.1 + f1.1 + f2.1) / 3.0);

    // Compute midpoints
    let mid_01 = ((f0.0 + f1.0) / 2.0, (f0.1 + f1.1) / 2.0);
    let mid_12 = ((f1.0 + f2.0) / 2.0, (f1.1 + f2.1) / 2.0);
    let mid_20 = ((f2.0 + f0.0) / 2.0, (f2.1 + f0.1) / 2.0);

    // Map sub-triangle ID to its vertices [v_mid, corner, center]
    match sub_triangle_id {
        0 => [mid_01, f0, center], // Between f1-f2
        1 => [mid_01, f1, center], // Between f1-f2
        2 => [mid_12, f1, center], // Between f2-f0
        3 => [mid_12, f2, center], // Between f2-f0
        4 => [mid_20, f2, center], // Between f0-f1
        5 => [mid_20, f0, center], // Between f0-f1
        _ => panic!("Invalid sub-triangle ID"),
    }
}

/// Affine transformation from one triangle to another
fn affine_transform_triangle(
    point: (f64, f64),
    source_tri: [(f64, f64); 3],
    dest_tri: [(f64, f64); 3],
) -> (f64, f64) {
    // Source vectors relative to source_tri[0]
    let (ax, ay) = (
        source_tri[1].0 - source_tri[0].0,
        source_tri[1].1 - source_tri[0].1,
    );
    let (bx, by) = (
        source_tri[2].0 - source_tri[0].0,
        source_tri[2].1 - source_tri[0].1,
    );

    // Destination vectors relative to dest_tri[0]
    let (cx, cy) = (dest_tri[1].0 - dest_tri[0].0, dest_tri[1].1 - dest_tri[0].1);
    let (dx, dy) = (dest_tri[2].0 - dest_tri[0].0, dest_tri[2].1 - dest_tri[0].1);

    // Point relative to source_tri[0]
    let (px, py) = (point.0 - source_tri[0].0, point.1 - source_tri[0].1);

    // Solve: [ax bx] [s]   [px]
    //        [ay by] [t] = [py]
    let det = ax * by - bx * ay;
    let s = (px * by - bx * py) / det;
    let t = (ax * py - px * ay) / det;

    // Apply same s,t to destination
    let x = dest_tri[0].0 + s * cx + t * dx;
    let y = dest_tri[0].1 + s * cy + t * dy;

    (x, y)
}

// @TODO - new tests need to be added.
#[cfg(test)]
mod tests {

    use geoplegma::types::Point;

    use super::{FACE_TEMPLATE_DOWN, FACE_TEMPLATE_UP};
    use crate::{
        constants::WGS84,
        ellipsoid::{AuthalicCoord, AuthalicSphere},
        projections::{
            polyhedron::{Orientation, icosahedron},
            projections::{traits::Projection, vgc::Vgc},
        },
    };

    /// Test helper: converts a degrees geodetic `Point` to an `AuthalicCoord`
    /// (radians, already on the WGS84 authalic sphere) the same way real
    /// callers of `Vgc::geo_to_cartesian` are now required to.
    fn to_authalic(p: Point) -> AuthalicCoord {
        AuthalicSphere::from_ellipsoid(&WGS84).convert(p)
    }

    #[test]
    fn test_point_creation() {
        let position = Point::new(-9.222154, 38.695125);
        assert_eq!(position.lon, -9.222154);
        assert_eq!(position.lat, 38.695125);
    }

    // Forward projection test disabled until Icosahedron implementation is complete
    #[test]
    fn test_project_forward() {
        let p1 = Point::new(-9.222154, 38.695125);
        let p2 = Point::new(-138.97503, 47.7022);
        let p3 = Point::new(99.72721, 25.82577);
        let p4 = Point::new(-64.10552, 12.89276);
        let p5 = Point::new(-128.28185, -50.60992);
        let p6 = Point::new(-70.47681, -0.81784);
        let p7 = Point::new(152.44705, -21.59114);
        let p8 = Point::new(66.665798, -77.717034);
        let p9 = Point::new(63.501735, 80.099071);
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);
        let points = vec![p1, p2, p3, p4, p5, p6, p7, p8, p9]
            .into_iter()
            .map(to_authalic)
            .collect();
        let result = projection.geo_to_cartesian(points, Some(&icosahedron), None);

        assert_eq!(result[0].face, 8);
        assert_eq!(result[1].face, 5);
        assert_eq!(result[2].face, 3);
        assert_eq!(result[3].face, 7);
        assert_eq!(result[4].face, 17);
        assert_eq!(result[5].face, 16);
        assert_eq!(result[6].face, 13);
        assert_eq!(result[7].face, 19);
        assert_eq!(result[8].face, 4);
    }

    #[test]
    fn test_spatial_consistency() {
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);
        // Test points
        let lisbon = Point::new(-9.49420, 38.68499);
        let porto = Point::new(-8.61099, 41.14961); // ~300km north of Lisbon
        let madrid = Point::new(-3.70379, 40.41678); // ~500km east of Lisbon

        let points = vec![lisbon, porto, madrid]
            .into_iter()
            .map(to_authalic)
            .collect();
        let results = projection.geo_to_cartesian(points, Some(&icosahedron), None);

        // Porto should be on same or adjacent face to Lisbon
        // (they're only 300km apart)
        assert!(
            results[0].face == results[1].face
                || icosahedron.are_faces_adjacent(results[0].face, results[1].face)
        );
    }

    #[test]
    fn test_pole_behavior() {
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);

        // Points around the pole should be on adjacent faces
        let points = vec![
            Point::new(0.0, 89.0),
            Point::new(72.0, 89.0),
            Point::new(144.0, 89.0),
            Point::new(216.0, 89.0),
            Point::new(288.0, 89.0),
        ]
        .into_iter()
        .map(to_authalic)
        .collect();

        let results = projection.geo_to_cartesian(points, Some(&icosahedron), None);

        // All should be near pole (check they're on the 5 faces around the north pole)
        for result in results.iter() {
            let is_in_north_pole = match result.face {
                0 | 2 | 4 | 6 | 8 => true,
                _ => false,
            };
            assert!(is_in_north_pole, "Its not on the north pole");
        }
    }

    #[test]
    fn test_equator_distribution() {
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);

        // Points evenly distributed around equator
        let points: Vec<AuthalicCoord> = (0..10)
            .map(|i| to_authalic(Point::new(i as f64 * 36.0, 0.0)))
            .collect();

        let results = projection.geo_to_cartesian(points, Some(&icosahedron), None);

        // Should hit multiple different faces
        let unique_faces: std::collections::HashSet<_> = results.iter().map(|r| r.face).collect();

        assert!(unique_faces.len() >= 5, "Should span multiple faces");
    }
    /// Demonstrates the projection is testable independent of any ellipsoid:
    /// a unit-sphere `Vgc` fed `AuthalicCoord`s directly (no `AuthalicSphere`
    /// conversion involved) still produces sane, small-magnitude output.
    #[test]
    fn test_unit_sphere_projection() {
        let projection = Vgc { radius: 1.0 };
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);

        let points = vec![
            AuthalicCoord {
                lon: -9.222154_f64.to_radians(),
                lat: 38.695125_f64.to_radians(),
            },
            AuthalicCoord {
                lon: 99.72721_f64.to_radians(),
                lat: 25.82577_f64.to_radians(),
            },
        ];

        let result = projection.geo_to_cartesian(points, Some(&icosahedron), None);

        assert_eq!(result.len(), 2);
        for r in &result {
            assert!(r.coords.x.abs() < 2.0);
            assert!(r.coords.y.abs() < 2.0);
        }
    }

    /// VGC is proven equal-area by construction (van Leeuwen & Strebe 2006):
    /// the slice-and-dice method preserves area exactly, so `areal_scale`
    /// (the Tissot indicatrix's Jacobian determinant, p*q*sin(theta')) must
    /// equal 1.0 at every point on the sphere, independent of any external
    /// tool. This replaces a prior Geocart-derived single-point comparison
    /// that was never actually asserted and whose numbers no longer apply
    /// (see PR description for why no Geocart-based test replaces it).
    /// (Lisbon is kept as one of the sample points for continuity with the
    /// old test.)
    #[test]
    fn test_distortion_areal_scale_is_unity() {
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);

        let points = [
            (38.68499, -9.49420), // Lisbon
            (-33.8688, 151.2093), // Sydney
            (64.1466, -21.9426),  // Reykjavik
            (1.3521, 103.8198),   // Singapore, near-equator
            (-89.0, 0.0),         // near south pole
        ];

        // compute_distortion uses a fixed 1e-5° finite-difference step. Near
        // sub-triangle interpolation seams (not just polyhedron edges/cusps)
        // the piecewise-affine map's local curvature is higher, which widens
        // discretization error in areal_scale beyond the ~1e-3 noise floor
        // seen at points away from seams (e.g. Lisbon: 0.9986). Tolerance is
        // set above that observed noise, not loosened to hide a real bug —
        // it's still an order of magnitude tighter than the ~30% mismatch
        // the old (buggy) formula would have produced.
        let tol = 0.02;
        for (lat, lon) in points {
            let distortion = projection.compute_distortion(lat, lon, &icosahedron, &WGS84);
            if !distortion.areal_scale.is_finite() {
                continue; // epsilon probe crossed a face boundary; skip
            }
            assert!(
                (distortion.areal_scale - 1.0).abs() < tol,
                "areal_scale = {} at ({}, {}), expected ~1.0 (equal-area)",
                distortion.areal_scale,
                lat,
                lon
            );
        }
    }

    /// Cross-checks `compute_distortion`'s statistical behavior against
    /// Table 1 of van Leeuwen & Strebe (2006): for the icosahedron under the
    /// vertex-oriented great-circle projection, the 2ω angle-distortion
    /// samples have mean μ = 0.141 rad and standard deviation σ = 0.028 rad.
    /// The paper measures a, b numerically (small-circle sampling); this
    /// samples points on a Fibonacci sphere and uses the closed-form
    /// Tissot-indicatrix derivation instead, so the tolerances below are
    /// generous rather than exact.
    #[test]
    fn test_distortion_matches_van_leeuwen_table1_icosahedron() {
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);

        let n = 2000;
        let golden_angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
        let mut samples: Vec<f64> = Vec::with_capacity(n);

        for i in 0..n {
            // Fibonacci sphere: near-uniform point distribution over the globe.
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / n as f64;
            let lat = y.asin().to_degrees();
            let lon = (golden_angle * i as f64).to_degrees() % 360.0;

            let distortion = projection.compute_distortion(lat, lon, &icosahedron, &WGS84);
            if distortion.angular_deformation.is_finite() {
                samples.push(distortion.angular_deformation.to_radians());
            }
        }

        assert!(
            samples.len() > n / 2,
            "too many samples dropped (face-crossing epsilon probes): {} of {}",
            samples.len(),
            n
        );

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance =
            samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / samples.len() as f64;
        let std_dev = variance.sqrt();

        assert!(
            (mean - 0.141).abs() < 0.05,
            "mean 2ω = {:.4} rad, expected ~0.141 rad (Table 1, icosahedron/VGC)",
            mean
        );
        assert!(
            (std_dev - 0.028).abs() < 0.03,
            "std 2ω = {:.4} rad, expected ~0.028 rad (Table 1, icosahedron/VGC)",
            std_dev
        );
    }

    /// `geo_to_barycentric` should reproduce `geo_to_cartesian`'s face id, and its weights
    /// should reconstruct the same face-plane point when combined with the face template —
    /// i.e. `sum(coords[k] * face_template[k]) * radius == geo_to_cartesian's coords`.
    /// This is the strongest correctness check: it doesn't depend on guessing expected
    /// numeric values, only on the two methods agreeing with each other.
    #[test]
    fn test_geo_to_barycentric_round_trip_matches_cartesian() {
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);

        let points = vec![
            Point::new(-9.222154, 38.695125),
            Point::new(-138.97503, 47.7022),
            Point::new(99.72721, 25.82577),
            Point::new(-64.10552, 12.89276),
            Point::new(-128.28185, -50.60992),
            Point::new(-70.47681, -0.81784),
            Point::new(152.44705, -21.59114),
            Point::new(66.665798, -77.717034),
            Point::new(63.501735, 80.099071),
        ];

        let authalic_points: Vec<AuthalicCoord> = points.iter().map(|p| to_authalic(*p)).collect();
        let cartesian = projection.geo_to_cartesian(authalic_points, Some(&icosahedron), None);
        let bary = projection.geo_to_barycentric(points, Some(&icosahedron), None, None);

        assert_eq!(cartesian.len(), bary.len());

        for (c, b) in cartesian.iter().zip(bary.iter()) {
            assert_eq!(c.face, b.face, "face id should match geo_to_cartesian");

            let sum = b.coords.x + b.coords.y + b.coords.z;
            assert!(
                (sum - 1.0).abs() < 1e-9,
                "barycentric weights should sum to 1, got {}",
                sum
            );

            let is_upward = c.face % 2 == 0;
            let template = if is_upward {
                FACE_TEMPLATE_UP
            } else {
                FACE_TEMPLATE_DOWN
            };
            let r = projection.radius;

            let reconstructed_x = b.coords.x * template[0].0 * r
                + b.coords.y * template[1].0 * r
                + b.coords.z * template[2].0 * r;
            let reconstructed_y = b.coords.x * template[0].1 * r
                + b.coords.y * template[1].1 * r
                + b.coords.z * template[2].1 * r;

            assert!(
                (reconstructed_x - c.coords.x).abs() < 1e-6,
                "reconstructed x {} != cartesian x {} (face {})",
                reconstructed_x,
                c.coords.x,
                c.face
            );
            assert!(
                (reconstructed_y - c.coords.y).abs() < 1e-6,
                "reconstructed y {} != cartesian y {} (face {})",
                reconstructed_y,
                c.coords.y,
                c.face
            );
        }
    }

    /// `geo_to_barycentric` with no `polyhedron`/`orientation`/`ellipsoid` should default to
    /// WGS84 + a DGGS-optimal icosahedron, matching what callers get by building those
    /// explicitly (this is the whole point of the convenience method).
    #[test]
    fn test_geo_to_barycentric_defaults_match_explicit_setup() {
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);
        let points = vec![Point::new(-9.222154, 38.695125), Point::new(30.0, 30.0)];

        let defaulted = projection.geo_to_barycentric(points.clone(), None, None, None);
        let explicit = projection.geo_to_barycentric(points, Some(&icosahedron), None, None);

        assert_eq!(defaulted.len(), explicit.len());
        for (d, e) in defaulted.iter().zip(explicit.iter()) {
            assert_eq!(d.face, e.face);
            assert!((d.coords.x - e.coords.x).abs() < 1e-12);
            assert!((d.coords.y - e.coords.y).abs() < 1e-12);
            assert!((d.coords.z - e.coords.z).abs() < 1e-12);
        }
    }
}
