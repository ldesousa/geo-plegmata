// Copyright 2025 contributors to the GeoPlegmata project.
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

//! Geographic coordinate utility functions
//! 
//! Functions for working with geographic coordinates, including validation,
//! normalization, and coordinate system conversions.

use geoplegma::types::Point;

use crate::{models::vector_3d::Vector3D, constants::Tolerance};

/// Convert geographic coordinates to 3D Cartesian coordinates on unit sphere
/// 
/// Transforms geographic coordinates (longitude, latitude in radians) to
/// 3D Cartesian coordinates on a unit sphere. This is commonly used
/// in spherical projections and discrete global grid systems.
/// 
/// # Arguments
/// * `cartesian` - Geographic point with longitude as lon and latitude as lat in radians
/// 
/// # Returns
/// Vector3D representing the point on the unit sphere
/// 
/// # Example
/// ```
/// use geo::Point;
/// use gp_proj::utils::geo_to_cartesian;
/// 
/// let cartesian = Point::new(0.0, 0.0); // Equator at prime meridian (in radians)
/// let result = geo_to_cartesian(&cartesian);
/// assert!((result.x - 1.0).abs() < 1e-10);
/// assert!(result.y.abs() < 1e-10);
/// assert!(result.z.abs() < 1e-10);
/// ```
pub fn geo_to_cartesian(cartesian: &Point) -> Vector3D {
    let lat_rad = cartesian.lat;
    let lon_rad = cartesian.lon;
    let cos_lat = lat_rad.cos();
    Vector3D {
        x: cos_lat * lon_rad.cos(),
        y: cos_lat * lon_rad.sin(),
        z: lat_rad.sin(),
    }
}

/// Normalize longitude to [-180, 180] range
/// 
/// Ensures longitude values are within the standard range by wrapping
/// values that exceed the bounds. This is essential for geographic
/// calculations that assume normalized coordinates.
/// 
/// # Arguments
/// * `lon` - Longitude in degrees (can be any value)
/// 
/// # Returns
/// Normalized longitude in range [-180, 180] degrees
/// 
/// # Example
/// ```
/// use gp_proj::utils::normalize_longitude;
/// 
/// assert_eq!(normalize_longitude(190.0), -170.0);
/// assert_eq!(normalize_longitude(-190.0), 170.0);
/// assert_eq!(normalize_longitude(0.0), 0.0);
/// ```
pub fn normalize_longitude(lon: f64) -> f64 {
    let mut normalized = lon % 360.0;
    if normalized > 180.0 {
        normalized -= 360.0;
    } else if normalized <= -180.0 {
        normalized += 360.0;
    }
    if normalized == -180.0 { 180.0 } else { normalized }
}

/// Create validated geographic point
/// 
/// Creates a geographic point after validating that coordinates are within
/// valid ranges. This prevents invalid coordinates from propagating through
/// the system and causing calculation errors.
/// 
/// # Arguments
/// * `lon` - Longitude in degrees, must be in range [-180, 180]
/// * `lat` - Latitude in degrees, must be in range [-90, 90]
/// 
/// # Returns
/// * `Ok(Point)` - Valid geographic point
/// * `Err(String)` - Error message describing the validation failure
/// 
/// # Example
/// ```
/// use gp_proj::utils::create_point;
/// 
/// let valid = create_point(45.0, 30.0);
/// assert!(valid.is_ok());
/// 
/// let invalid = create_point(200.0, 30.0); // Invalid longitude
/// assert!(invalid.is_err());
/// ```
pub fn create_point(lon: f64, lat: f64) -> Result<Point, String> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(format!("Latitude must be in range [-90, 90], got {}", lat));
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err(format!("Longitude must be in range [-180, 180], got {}", lon));
    }
    Ok(Point::new(lon, lat))
}

/// Create validated geographic point with automatic longitude normalization
/// 
/// Similar to create_point, but automatically normalizes longitude values
/// that are outside the standard range instead of returning an error.
/// 
/// # Arguments
/// * `lon` - Longitude in degrees (will be normalized)
/// * `lat` - Latitude in degrees, must be in range [-90, 90]
/// 
/// # Returns
/// * `Ok(Point)` - Valid geographic point with normalized longitude
/// * `Err(String)` - Error message if latitude is invalid
pub fn create_point_normalized(lon: f64, lat: f64) -> Result<Point, String> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(format!("Latitude must be in range [-90, 90], got {}", lat));
    }
    let normalized_lon = normalize_longitude(lon);
    Ok(Point::new(normalized_lon, lat))
}

/// Check if two points are approximately equal within tolerance
/// 
/// Compares two geographic points using a specified tolerance to account
/// for floating-point precision errors. Uses Euclidean distance in the
/// coordinate space (not great circle distance).
/// 
/// # Arguments
/// * `p1` - First point to compare
/// * `p2` - Second point to compare  
/// * `tolerance` - Optional tolerance in degrees (uses default if None)
/// 
/// # Returns
/// True if points are within tolerance, false otherwise
/// 
/// # Example
/// ```
/// use geo::Point;
/// use gp_proj::utils::points_approx_eq;
/// 
/// let p1 = Point::new(1.0, 2.0);
/// let p2 = Point::new(1.0001, 2.0001);
/// 
/// assert!(points_approx_eq(&p1, &p2, Some(0.001)));
/// assert!(!points_approx_eq(&p1, &p2, Some(0.00001)));
/// ```
pub fn points_approx_eq(p1: &Point, p2: &Point, tolerance: Option<f64>) -> bool {
    let tol = tolerance.unwrap_or(Tolerance::COORDINATE);
    (p1.lon - p2.lon).abs() < tol && (p1.lat - p2.lat).abs() < tol
}

/// Calculate great circle distance between two points
/// 
/// Uses the haversine formula to calculate the shortest distance between
/// two points on the Earth's surface. Returns distance in meters.
/// 
/// # Arguments
/// * `p1` - First geographic point
/// * `p2` - Second geographic point
/// 
/// # Returns
/// Distance in meters along the great circle
/// 
/// # Example
/// ```
/// use geo::Point;
/// use gp_proj::utils::great_circle_distance;
/// 
/// let p1 = Point::new(0.0, 0.0);
/// let p2 = Point::new(1.0, 0.0);
/// let distance = great_circle_distance(&p1, &p2);
/// assert!(distance > 111000.0); // Approximately 111 km per degree at equator
/// ```
pub fn great_circle_distance(p1: &Point, p2: &Point) -> f64 {
    use crate::constants::WGS84;
    
    let lat1 = p1.lat.to_radians();
    let lat2 = p2.lat.to_radians();
    let dlat = lat2 - lat1;
    let dlon = (p2.lon - p1.lon).to_radians();
    
    let a = (dlat / 2.0).sin().powi(2) + 
            lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    WGS84::AUTHALIC_RADIUS * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_to_cartesian() {
        // Test point at (0, 0) radians - should be at (1, 0, 0) on unit sphere
        let point = Point::new(0.0, 0.0);
        let result = geo_to_cartesian(&point);
        
        assert!((result.x - 1.0).abs() < 1e-10);
        assert!(result.y.abs() < 1e-10);
        assert!(result.z.abs() < 1e-10);
        
        // Test that result is on unit sphere
        assert!((result.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude() {
        assert_eq!(normalize_longitude(0.0), 0.0);
        assert_eq!(normalize_longitude(180.0), 180.0);
        assert_eq!(normalize_longitude(-180.0), 180.0);
        assert_eq!(normalize_longitude(190.0), -170.0);
        assert_eq!(normalize_longitude(-190.0), 170.0);
        assert_eq!(normalize_longitude(360.0), 0.0);
        assert_eq!(normalize_longitude(-360.0), 0.0);
    }

    #[test]
    fn test_create_point_validation() {
        // Valid coordinates
        assert!(create_point(0.0, 0.0).is_ok());
        assert!(create_point(180.0, 90.0).is_ok());
        assert!(create_point(-180.0, -90.0).is_ok());
        
        // Invalid latitude
        assert!(create_point(0.0, 91.0).is_err());
        assert!(create_point(0.0, -91.0).is_err());
        
        // Invalid longitude
        assert!(create_point(181.0, 0.0).is_err());
        assert!(create_point(-181.0, 0.0).is_err());
    }

    #[test]
    fn test_create_point_normalized() {
        // Should normalize longitude but validate latitude
        let result = create_point_normalized(190.0, 45.0).unwrap();
        assert_eq!(result.lon, -170.0);
        assert_eq!(result.lat, 45.0);
        
        // Should still reject invalid latitude
        assert!(create_point_normalized(0.0, 91.0).is_err());
    }

    #[test]
    fn test_points_approx_eq() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(1.0001, 2.0001);
        
        assert!(points_approx_eq(&p1, &p2, Some(0.001)));
        assert!(!points_approx_eq(&p1, &p2, Some(0.00001)));
        
        // Test with default tolerance
        let p3 = Point::new(1.0, 2.0);
        let p4 = Point::new(1.0, 2.0);
        assert!(points_approx_eq(&p3, &p4, None));
    }

    #[test]
    fn test_great_circle_distance() {
        // Distance from equator/prime meridian to 1 degree east should be ~111 km
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(1.0, 0.0);
        let distance = great_circle_distance(&p1, &p2);
        
        assert!(distance > 110000.0);
        assert!(distance < 112000.0);
        
        // Distance from a point to itself should be zero
        let distance_zero = great_circle_distance(&p1, &p1);
        assert!(distance_zero < 1e-10);
    }
}