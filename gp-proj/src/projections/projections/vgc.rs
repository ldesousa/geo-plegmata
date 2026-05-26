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
    constants::WGS84, ellipsoid::{AuthalicCoord, AuthalicSphere}, models::vector_3d::Vector3D, projections::{
        layout::traits::Layout,
        polyhedron::{ArcLengths, Polyhedron},
        projections::traits::{DistortionMetrics, ForwardCartesian, Projection},
    },
    utils::shape::triangle,
};
use geo::{Coord, Point};

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
pub struct Vgc{
    pub radius: f64,
}

impl Default for Vgc{
    fn default() -> Self {
        Self{
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
                        face: index + 1,
                        sub_triangle_id,
                    });

                    // in case the point is on the edge of two faces, we return the first face.
                    break;
                }
            }
        }
        out
    }

    fn cartesian_to_geo(
        &self,
        positions: Vec<ForwardCartesian>,
        polyhedron: Option<&Polyhedron>,
    ) -> Vec<Point> {
        let mut out: Vec<Point> = vec![];
        let polyhedron = polyhedron.unwrap();
        let coef_fourier_auth_to_geod =
            Self::fourier_coefficients(KarneyCoefficients::AUTHALIC_TO_GEODETIC);
        let r = 6371007.181_f64;

        for position in positions {
            let face = usize::from(position.face);
            let sub_triangle_id = position.sub_triangle_id;

            // STEP 1: divide out radius
            let p_x_face = position.coords.x / r;
            let p_y_face = position.coords.y / r;

            // STEP 2: get face template and sub-triangle vertices in face space
            let is_upward = face % 2 == 0;
            let face_template = if is_upward {
                FACE_TEMPLATE_UP
            } else {
                FACE_TEMPLATE_DOWN
            };
            let sub_vertices_in_face =
                get_subtriangle_vertices_in_face(sub_triangle_id, face_template);

            // STEP 3: inverse affine — face space → sub-triangle template space
            let (p_x_local, p_y_local) = affine_transform_triangle(
                (p_x_face, p_y_face),
                sub_vertices_in_face, // ← source and dest swapped vs forward
                SUB_TRIANGLE_TEMPLATE,
            );

            // STEP 4: recover xy and uv from p_local
            // p_local = B + xy * (D - B)  where D = C + uv * (A - C)
            // B = SUB_TRIANGLE_TEMPLATE[1], A = SUB_TRIANGLE_TEMPLATE[0], C = SUB_TRIANGLE_TEMPLATE[2]
            let b = SUB_TRIANGLE_TEMPLATE[1];
            let a = SUB_TRIANGLE_TEMPLATE[0];
            let c = SUB_TRIANGLE_TEMPLATE[2];

            // xy = distance(p_local, B) / distance(D, B)
            // First recover xy from the direction B→p_local vs B→A direction
            let bp_x = p_x_local - b.0;
            let bp_y = p_y_local - b.1;
            let ba_x = a.0 - b.0;
            let ba_y = a.1 - b.1;

            // xy is the parameter along B→D, recover it via projection
            let xy = (bp_x * ba_x + bp_y * ba_y) / (ba_x * ba_x + ba_y * ba_y);

            // D = p_local interpolated back: D = B + xy*(D-B) → D = p_local/xy + B*(1-1/xy)...
            // simpler: D = (p_local - B*(1-xy)) / xy...
            // Actually: p_local = B + xy*(D-B) → D = B + (p_local-B)/xy
            let pd_x = b.0 + (p_x_local - b.0) / xy;
            let pd_y = b.1 + (p_y_local - b.1) / xy;

            // D = C + uv*(A-C) → uv = (D-C)/(A-C)
            let ac_x = a.0 - c.0;
            let ac_y = a.1 - c.1;
            let uv = ((pd_x - c.0) * ac_x + (pd_y - c.1) * ac_y) / (ac_x * ac_x + ac_y * ac_y);

            // STEP 5: recover arc lengths from xy and uv
            // get sub-triangle 3D vertices
            let sub_triangle_3d = triangle_by_id(polyhedron, face, sub_triangle_id);
            let ArcLengths {
                ab, bc, ac: ac_len, ..
            } = polyhedron.arc_lengths(sub_triangle_3d, Vector3D::zero());

            // inverse slice_and_dice: recover ap and bp
            let [ap, bp] = inverse_slice_and_dice(ac_len, ab, bc, xy, uv);

            // STEP 6: reconstruct 3D point from sub-triangle vertices and arc lengths
            let point_p = reconstruct_point(sub_triangle_3d, ap, bp);

            // STEP 7: 3D → authalic lat/lon → geodetic lat/lon
            let lat_auth = point_p.z.asin();
            let lon = point_p.y.atan2(point_p.x);
            let lat_geod = Self::lat_authalic_to_geodetic(lat_auth, &coef_fourier_auth_to_geod);

            out.push(Point::new(lon.to_degrees(), lat_geod.to_degrees()));
        }
        out
    }

    // @TODO - Needs to be reviewed
    // Calculate distortion and compare with Geocart values
    fn compute_distortion(&self, lat: f64, lon: f64, polyhedron: &Polyhedron) -> DistortionMetrics {
        let epsilon = 1e-5_f64; // degrees
        let sphere = AuthalicSphere::from_ellipsoid(&WGS84);
        let to_authalic = |lon: f64, lat: f64| sphere.convert(Point::new(lon, lat));

        let center_xy = 
            &self.geo_to_cartesian(vec![to_authalic(lon, lat)], Some(polyhedron), None)[0];
        let north_xy =
            &self.geo_to_cartesian(vec![to_authalic(lon, lat + epsilon)], Some(polyhedron), None)[0];
        let east_xy = 
            &self.geo_to_cartesian(vec![to_authalic(lon + epsilon, lat)], Some(polyhedron), None)[0];
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

        // WGS84 radii of curvature (meters/radian)
        let a = 6378137.0_f64;
        let e2 = 0.00669437999014_f64;
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

        // Tissot: a and b are semi-axes of the indicatrix ellipse
        let p = (e.powi(2) + f.powi(2)).sqrt();
        let q = (g.powi(2) + h_.powi(2)).sqrt();
        let t = e * g + f * h_;

        let a_tissot = ((p + q).powi(2)
            - 2.0 * (e * h_ - f * g).abs() * (1.0 - (t / (p * q)).powi(2)).sqrt())
        .sqrt()
            / 2.0_f64.sqrt();
        let b_tissot = ((p - q).powi(2)
            + 2.0 * (e * h_ - f * g).abs() * (1.0 - (t / (p * q)).powi(2)).sqrt())
        .sqrt()
            / 2.0_f64.sqrt();

        let areal_scale = (e * h_ - f * g).abs();
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

/// Get the 3D sub-triangle vertices directly from face and sub-triangle ID,
/// without needing the point. Returns [mid, corner, center] in 3D.
pub fn triangle_by_id(polyhedron: &Polyhedron, face: usize, sub_triangle_id: u8) -> [Vector3D; 3] {
    let verts = polyhedron.face_vertices(face).unwrap();
    let [v0, v1, v2] = [verts[0], verts[1], verts[2]];
    let center = polyhedron.face_center(face);

    let mid_01 = (v0 + v1).normalize();
    let mid_12 = (v1 + v2).normalize();
    let mid_20 = (v2 + v0).normalize();

    // Same order as get_subtriangle_vertices_in_face: [mid, corner, center]
    match sub_triangle_id {
        0 => [mid_01, v0, center],
        1 => [mid_01, v1, center],
        2 => [mid_12, v1, center],
        3 => [mid_12, v2, center],
        4 => [mid_20, v2, center],
        5 => [mid_20, v0, center],
        _ => panic!("Invalid sub_triangle_id: {}", sub_triangle_id),
    }
}

pub fn inverse_slice_and_dice(
    ac: f64,
    ab: f64,
    bc: f64,
    xy_target: f64,
    uv_target: f64,
) -> [f64; 2] {
    // Numerical inversion via Newton's method
    // We search for (ap, bp) such that slice_and_dice(ac,ab,bc,ap,bp) = [xy_target, uv_target]

    // Initial guess: linear approximation
    let mut ap = xy_target * ab;
    let mut bp = uv_target * bc;

    let eps = 1e-10_f64;
    let h = 1e-7_f64; // finite difference step

    for _ in 0..50 {
        let [xy, uv] = slice_and_dice(ac, ab, bc, ap, bp);

        let r_xy = xy - xy_target;
        let r_uv = uv - uv_target;

        // converged?
        if r_xy.abs() < eps && r_uv.abs() < eps {
            break;
        }

        // Jacobian via finite differences
        let [xy_ap, uv_ap] = slice_and_dice(ac, ab, bc, ap + h, bp);
        let [xy_bp, uv_bp] = slice_and_dice(ac, ab, bc, ap, bp + h);

        let d_xy_ap = (xy_ap - xy) / h;
        let d_uv_ap = (uv_ap - uv) / h;
        let d_xy_bp = (xy_bp - xy) / h;
        let d_uv_bp = (uv_bp - uv) / h;

        // 2x2 Jacobian:
        // | d_xy_ap  d_xy_bp | | delta_ap |   | r_xy |
        // | d_uv_ap  d_uv_bp | | delta_bp | = | r_uv |

        let det = d_xy_ap * d_uv_bp - d_xy_bp * d_uv_ap;
        if det.abs() < 1e-14 {
            break; // singular, can't continue
        }

        let delta_ap = (r_xy * d_uv_bp - r_uv * d_xy_bp) / det;
        let delta_bp = (d_xy_ap * r_uv - d_uv_ap * r_xy) / det;

        ap = (ap - delta_ap).clamp(0.0, ab);
        bp = (bp - delta_bp).clamp(0.0, bc);
    }

    [ap, bp]
}

/// Reconstruct a 3D unit vector on the sphere given a sub-triangle
/// [mid=A, corner=B, center=C] and arc lengths ap (B→P) and bp (along AC direction).
///
/// Strategy:
/// 1. Find point D on AC at parameter uv: D = slerp(A, C, uv)
/// 2. Find point P on BD at parameter xy: P = slerp(B, D, xy)
pub fn reconstruct_point(sub_triangle: [Vector3D; 3], ap: f64, bp: f64) -> Vector3D {
    let [a, b, c] = sub_triangle; // [mid, corner, center]

    // We need uv and xy, but we have ap and bp (arc lengths)
    // ap = xy * ab, bp = uv * bc
    let ab = a.dot(b);
    let bc = b.dot(c);

    let xy = (ap / ab).clamp(0.0, 1.0);
    let uv = (bp / bc).clamp(0.0, 1.0);

    // D = slerp(A, C, uv)
    let d = slerp(a, c, uv);

    // P = slerp(B, D, xy)
    slerp(b, d, xy)
}

/// Spherical linear interpolation between two unit vectors at parameter t ∈ [0,1].
pub fn slerp(a: Vector3D, b: Vector3D, t: f64) -> Vector3D {
    let dot = a.dot(b).clamp(-1.0, 1.0);
    let omega = dot.acos();

    if omega.abs() < 1e-10 {
        // Vectors are nearly identical, linear interpolation is fine
        return (a * (1.0 - t) + b * t).normalize();
    }

    let sin_omega = omega.sin();
    let scale_a = ((1.0 - t) * omega).sin() / sin_omega;
    let scale_b = (t * omega).sin() / sin_omega;

    (a * scale_a + b * scale_b).normalize()
}

// @TODO - new tests need to be added.
#[cfg(test)]
mod tests {
    use geo::Point;

    use crate::{
        constants::WGS84,
        ellipsoid::{AuthalicCoord, AuthalicSphere},
        projections::{
            polyhedron::{icosahedron, Orientation},
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
        assert_eq!(position.x(), -9.222154);
        assert_eq!(position.y(), 38.695125);
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

        let points = vec![lisbon, porto, madrid].into_iter().map(to_authalic).collect();
        let results = projection.geo_to_cartesian(points, Some(&icosahedron), None);

        // Check they're on reasonable faces
        println!("Lisbon face: {}", results[0].face);
        println!("Porto face: {}", results[1].face);
        println!("Madrid face: {}", results[2].face);

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
        for (i, result) in results.iter().enumerate() {
            println!(
                "Point {} - Face: {}, Coords: {:?}",
                i, result.face, result.coords
            );
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

        println!("Unique faces at equator: {:?}", unique_faces);
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
            AuthalicCoord { lon: -9.222154_f64.to_radians(), lat: 38.695125_f64.to_radians() },
            AuthalicCoord { lon: 99.72721_f64.to_radians(), lat: 25.82577_f64.to_radians() },
        ];

        let result = projection.geo_to_cartesian(points, Some(&icosahedron), None);

        assert_eq!(result.len(), 2);
        for r in &result {
            assert!(r.coords.x.abs() < 2.0);
            assert!(r.coords.y.abs() < 2.0);
        }
    }

    #[test]
    fn test_distortion() {
        let projection = Vgc::default();
        let icosahedron = icosahedron::new(Orientation::DGGS_OPTIMAL);
        let distortion = projection.compute_distortion(38.68499, -9.49420, &icosahedron);
        println!("h: {} (expected: 0.7580403)", distortion.h);
        println!("k: {} (expected: 1.333174)", distortion.k);
        println!(
            "Angular deformation: {}° (expected: 33.045°)",
            distortion.angular_deformation
        );
        println!(
            "Areal scale: {} (expected: ~1.0 for equal-area)",
            distortion.areal_scale
        );
    }
}
