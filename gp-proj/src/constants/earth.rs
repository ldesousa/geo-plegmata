// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

//! Earth ellipsoid and geodetic constants
//! 
//! Contains standardized parameters for the WGS84 ellipsoid and derived values
//! commonly used in geographic coordinate transformations and projections.

/// WGS84 ellipsoid constants and derived parameters
/// 
/// World Geodetic System 1984 is the standard coordinate system used by GPS
/// and most modern mapping applications. These constants define the shape
/// and size of the Earth approximation used in geodetic calculations.
pub struct WGS84;

impl WGS84 {
    /// Semi-major axis (equatorial radius) in meters
    /// 
    /// The longest radius of the WGS84 ellipsoid, measured from the center
    /// to the equator. This is the "a" parameter in ellipsoid definitions.
    pub const MAJOR_AXIS: f64 = 6378137.0;
    
    /// Semi-minor axis (polar radius) in meters
    /// 
    /// The shortest radius of the WGS84 ellipsoid, measured from the center
    /// to the pole. This is the "b" parameter in ellipsoid definitions.
    pub const MINOR_AXIS: f64 = 6356752.314245;
    
    /// Authalic sphere radius in meters
    /// 
    /// Radius of a sphere with the same surface area as the WGS84 ellipsoid.
    /// Used in equal-area projections and discrete global grid systems.
    /// Formula: R = √((a²b²)/(a²+b²)) where a,b are major,minor axes
    pub const AUTHALIC_RADIUS: f64 = 6371007.1809184747;
    
    /// First eccentricity squared (e²)
    /// 
    /// Measure of how much the ellipsoid deviates from a perfect sphere.
    /// Formula: e² = (a² - b²) / a²
    /// Used in coordinate transformations and map projections.
    pub const ECCENTRICITY_SQUARED: f64 = 0.006694379990141316;
    
    /// Flattening (f)
    /// 
    /// Ratio describing how "flattened" the ellipsoid is compared to a sphere.
    /// Formula: f = (a - b) / a
    /// The inverse flattening (1/f) for WGS84 is approximately 298.257223563
    pub const FLATTENING: f64 = 1.0 / 298.257223563;
    
    /// Third flattening (n)
    /// 
    /// Alternative flattening parameter used in some geodetic calculations.
    /// Formula: n = (a - b) / (a + b)
    /// Often used in series expansions for coordinate transformations.
    pub const THIRD_FLATTENING: f64 = Self::FLATTENING / (2.0 - Self::FLATTENING);
}


impl crate::ellipsoid::Ellipsoid for WGS84 {
    fn third_flattening(&self) -> f64 {
        Self::THIRD_FLATTENING
    }

    fn authalic_radius(&self) -> f64 {
        Self::AUTHALIC_RADIUS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgs84_constants_relationships() {
        // Test that derived constants are consistent
        let a = WGS84::MAJOR_AXIS;
        let b = WGS84::MINOR_AXIS;
        
        // Check flattening calculation
        let calculated_flattening = (a - b) / a;
        assert!((calculated_flattening - WGS84::FLATTENING).abs() < 1e-10);
        
        // Check eccentricity calculation
        let calculated_e_sq = (a * a - b * b) / (a * a);
        assert!((calculated_e_sq - WGS84::ECCENTRICITY_SQUARED).abs() < 1e-10);
        
        // Check that major axis is larger than minor axis
        assert!(WGS84::MAJOR_AXIS > WGS84::MINOR_AXIS);
        
        // Check that authalic radius is between major and minor axes
        assert!(WGS84::MINOR_AXIS < WGS84::AUTHALIC_RADIUS);
        assert!(WGS84::AUTHALIC_RADIUS < WGS84::MAJOR_AXIS);
    }
}