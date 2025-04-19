/// Radius
/// Derivation of the radius vector (R') and the Earth Radius (R)
/// RR = R' / R = (1 / (2 * sqrt(5)) + 1 / 6) * sqrt(PI * sqrt(3));
// pub const RR: f64 = 0.9449322893;
// pub const RR: f64 = 0.94449322893;
pub const RR: f64 = 0.9103832815095034;

/// Radius
/// Authalic sphere radius for WGS84 [m]
pub const AUTHALIC_EARTH_RADIUS: f64 = 6371007.1809184747;

/// Spherical Constant
/// Spherical distance in degrees (g) for the icosahedron, from center of polygon face to any of its vertices on the globe.
pub const SPHERICAL_DISTANCE: f64 = 37.37736814; // g

/// Spherical Constant
/// Plane angle in degrees (θ), between the radius vector (R´) to the center and adjacent edge of plane polygon.
pub const THETA: f64 = 30.0; // θ

pub const GOLDEN_RATIO_ICOSAHEDRON: f64 = 1.618;
