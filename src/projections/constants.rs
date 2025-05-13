/// Radius
/// Derivation of the radius vector (R') and the Earth Radius (R)
/// RR = R' / R = (1 / (2 * sqrt(5)) + 1 / 6) * sqrt(PI * sqrt(3));
// pub const RR: f64 = 0.9449322893;
// pub const RR: f64 = 0.94449322893;
pub const RR: f64 = 0.9103832815095034;

/// Radius
/// Authalic sphere radius for WGS84 [m]
pub const AUTHALIC_EARTH_RADIUS: f64 = 6371007.1809184747;

/// Elipsoide constants for WGS84
pub const ELIPSOID_MAJOR: f64 = 6378137.0;
pub const ELIPSOID_MINOR: f64 = 6356752.314245;

// Cξφ (A19) - coefficients to convert geodetic latitude to authalic latitude (Karney, 2023)
pub const COEF_AUTH_TO_GEOD_LAT: [f64; 21] = [
    -4.0 / 3.0,
    -4.0 / 45.0,
    88.0 / 315.0,
    538.0 / 4725.0,
    20824.0 / 467775.0,
    -44732.0 / 2837835.0,
    34.0 / 45.0,
    8.0 / 105.0,
    -2482.0 / 14175.0,
    -37192.0 / 467775.0,
    -12467764.0 / 212837625.0,
    -1532.0 / 2835.0,
    -898.0 / 14175.0,
    54968.0 / 467775.0,
    100320856.0 / 1915538625.0,
    6007.0 / 14175.0,
    24496.0 / 467775.0,
    -5884124.0 / 70945875.0,
    -23356.0 / 66825.0,
    -839792.0 / 19348875.0,
    570284222.0 / 1915538625.0,
];

// // Cφξ (A20) - coefficients to convert authalic latitude to geodetic latitude (Karney, 2023)
pub const COEF_GEOD_TO_AUTH_LAT: [f64; 21] = [
    4.0 / 3.0,
    4.0 / 45.0,
    -16.0 / 35.0,
    -2582.0 / 14175.0,
    60136.0 / 467775.0,
    28112932.0 / 212837625.0,
    46.0 / 45.0,
    152.0 / 945.0,
    -11966.0 / 14175.0,
    -21016.0 / 51975.0,
    251310128.0 / 638512875.0,
    3044.0 / 2835.0,
    3802.0 / 14175.0,
    -94388.0 / 66825.0,
    -8797648.0 / 10945935.0,
    6059.0 / 4725.0,
    41072.0 / 93555.0,
    -1472637812.0 / 638512875.0,
    768272.0 / 467775.0,
    -455935736.0 / 638512875.0,
    4210684958.0 / 1915538625.0,
];
