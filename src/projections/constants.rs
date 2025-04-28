/// Radius
/// Derivation of the radius vector (R') and the Earth Radius (R)
/// RR = R' / R = (1 / (2 * sqrt(5)) + 1 / 6) * sqrt(PI * sqrt(3));
// pub const RR: f64 = 0.9449322893;
// pub const RR: f64 = 0.94449322893;
pub const RR: f64 = 0.9103832815095034;

/// Radius
/// Authalic sphere radius for WGS84 [m]
pub const AUTHALIC_EARTH_RADIUS: f64 = 6371007.1809184747;