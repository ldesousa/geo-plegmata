// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::constants::DGGRS_SPECS;
use crate::error::DggrsError;
use crate::error::factory::DggrsUidError;
use std::convert::{From, TryFrom};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

impl Point {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    pub fn to_coord(&self) -> geo::Coord {
        geo::coord! { x: self.lon, y: self.lat }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Region {
    pub exterior: Vec<Point>,
}

impl Region {
    pub fn new(mut exterior: Vec<Point>) -> Self {
        if let Some(first) = exterior.first().copied() {
            if exterior.last().copied() != Some(first) {
                exterior.push(first);
            }
        }
        Self { exterior }
    }

    pub fn coords_count(&self) -> usize {
        self.exterior.len()
    }

    pub fn to_geo_polygon(&self) -> geo::Polygon<f64> {
        let coords: Vec<_> = self.exterior.iter().map(Point::to_coord).collect();
        geo::Polygon::new(geo::LineString::from(coords), vec![])
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BoundingBox {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

impl BoundingBox {
    pub fn new(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> Self {
        Self {
            min_lon,
            min_lat,
            max_lon,
            max_lat,
        }
    }

    pub const WORLD: Self = Self {
        min_lon: -180.0,
        min_lat: -90.0,
        max_lon: 180.0,
        max_lat: 90.0,
    };
}

// NOTE: The naming needs to be adjusted to the DGGRS Registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DggrsUid {
    ISEA3HDGGRID,
    IGEO7,
    H3,
    IVEA3H,
    ISEA3HDGGAL,
    IVEA9R,
    ISEA9R,
    RTEA3H,
    RTEA9R,
    IVEA7H,
    IVEA7H_Z7,
}

impl DggrsUid {
    #[inline]
    const fn idx(self) -> usize {
        match self {
            DggrsUid::ISEA3HDGGRID => 0,
            DggrsUid::IGEO7 => 1,
            DggrsUid::H3 => 2,
            DggrsUid::ISEA3HDGGAL => 3,
            DggrsUid::IVEA3H => 4,
            DggrsUid::ISEA9R => 5,
            DggrsUid::IVEA9R => 6,
            DggrsUid::RTEA3H => 7,
            DggrsUid::RTEA9R => 8,
            DggrsUid::IVEA7H => 9,
            DggrsUid::IVEA7H_Z7 => 10,
        }
    }

    #[inline]
    pub fn spec(self) -> &'static DggrsSpec {
        &DGGRS_SPECS[self.idx()]
    }
}

impl FromStr for DggrsUid {
    type Err = DggrsUidError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let needle = input.trim();

        for spec in DGGRS_SPECS.iter() {
            if needle == spec.id.to_string() {
                return Ok(spec.id);
            }
        }

        let candidates = DGGRS_SPECS.iter().map(|s| s.id).collect();
        Err(DggrsUidError::Unknown {
            input: input.to_string(),
            candidates,
        })
    }
}

impl fmt::Display for DggrsUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DggrsName {
    ISEA3H,
    IGEO7,
    H3,
    IVEA3H,
    IVEA9R,
    ISEA9R,
    RTEA3H,
    RTEA9R,
    IVEA7H,
    IVEA7H_Z7,
}
impl fmt::Display for DggrsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DggrsName::ISEA3H => "ISEA3H",
            DggrsName::IGEO7 => "IGEO7",
            DggrsName::H3 => "H3",
            DggrsName::IVEA3H => "IVEA3H",
            DggrsName::IVEA9R => "IVEA9R",
            DggrsName::ISEA9R => "ISEA9R",
            DggrsName::RTEA3H => "RTEA3H",
            DggrsName::RTEA9R => "RTEA9R",
            DggrsName::IVEA7H => "IVEA7H",
            DggrsName::IVEA7H_Z7 => "IVEA7H_Z7",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone)]
pub enum DggrsImplementation {
    Native,
    DGGRID,
    DGGAL,
    H3O,
}

impl fmt::Display for DggrsImplementation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DggrsImplementation::Native => "Native",
            DggrsImplementation::DGGRID => "DGGRID",
            DggrsImplementation::DGGAL => "DGGAL",
            DggrsImplementation::H3O => "H3O",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone)]
pub struct DggrsSpec {
    pub id: DggrsUid,
    pub name: DggrsName,
    pub tool: DggrsImplementation,
    pub title: &'static str,
    pub description: &'static str,
    pub uri: &'static str,
    pub crs: &'static str,
    pub aperture: u32,
    pub min_refinement_level: RefinementLevel,
    pub max_refinement_level: RefinementLevel,
    pub default_refinement_level: RefinementLevel,
    pub max_relative_depth: RelativeDepth,
    pub default_relative_depth: RelativeDepth,
}

#[derive(Debug, Clone, Default)]
pub struct Zone {
    pub id: ZoneId,
    pub region: Option<Region>,
    pub center: Option<Point>,
    pub vertex_count: Option<u32>,
    pub children: Option<Vec<ZoneId>>,
    pub neighbors: Option<Vec<ZoneId>>,
    pub area_sqm: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct Zones {
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ZoneId {
    StrId(String),
    HexId(HexString),
    IntId(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HexString(String);

impl HexString {
    pub fn new(s: &str) -> Result<Self, String> {
        if s.len() <= 16 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Self(s.to_string()))
        } else {
            Err("HexId can have a maximum length of 16 hexadecimal characters.".to_string())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HexString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ZoneId {
    /// 1 to 32 character ZoneID
    pub fn new_str(s: &str) -> Result<Self, DggrsError> {
        if (1..=32).contains(&s.len()) {
            Ok(ZoneId::StrId(s.to_string()))
        } else {
            Err(DggrsError::UnsupportedZoneIdFormat(format!(
                "StrId must be between 1 and 32 characters got '{}'",
                s
            )))
        }
    }

    /// Hexadecimal ZoneId
    pub fn new_hex(s: &str) -> Result<Self, DggrsError> {
        HexString::new(s)
            .map(ZoneId::HexId)
            .map_err(|e| DggrsError::InvalidHexId(e))
    }

    /// 64 bit Integer ZoneId
    pub fn new_int(id: u64) -> Self {
        ZoneId::IntId(id)
    }

    /// convert ZoneId::StrId as String
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ZoneId::StrId(s) => Some(s),
            _ => None,
        }
    }

    /// convert ZoneId::HexId as String
    pub fn as_hex(&self) -> Option<&HexString> {
        match self {
            ZoneId::HexId(h) => Some(h),
            _ => None,
        }
    }

    /// convert ZoneId::IntId as String
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            ZoneId::IntId(i) => Some(*i),
            _ => None,
        }
    }
}

impl FromStr for ZoneId {
    type Err = DggrsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // 1) pure decimal => Int
        if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
            if let Ok(v) = s.parse::<u64>() {
                return Ok(ZoneId::new_int(v));
            }
        }

        // 2) hex-looking => Hex (let HexString validate)
        let is_hexish = !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit());
        if is_hexish {
            if let Ok(h) = ZoneId::new_hex(s) {
                return Ok(h);
            }
            // fall through if it *looks* hex but failed HexString validation
        }

        // 4) fallback => Str
        ZoneId::new_str(s)
    }
}

impl Default for ZoneId {
    fn default() -> Self {
        ZoneId::HexId(HexString::new("0000000000000000").unwrap())
    }
}

impl fmt::Display for ZoneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneId::StrId(s) => write!(f, "{s}"),
            ZoneId::HexId(s) => write!(f, "{s}"),
            ZoneId::IntId(i) => write!(f, "{i}"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefinementLevel(i32);

impl RefinementLevel {
    pub fn new(value: i32) -> Result<Self, DggrsError> {
        if value < 0 {
            Err(DggrsError::DepthBelowZero(value))
        } else {
            Ok(Self(value))
        }
    }

    pub fn get(self) -> i32 {
        self.0
    }
    pub fn add(self, rd: RelativeDepth) -> Result<Self, DggrsError> {
        RefinementLevel::new(self.0 + rd.0)
    }

    pub const fn new_const(val: i32) -> Self {
        // trust that `val` is valid at compile time
        Self(val)
    }
}

// i32 → Depth (fallible)
impl TryFrom<i32> for RefinementLevel {
    type Error = DggrsError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        RefinementLevel::new(value)
    }
}

// u8 → Depth (infallible)
impl From<u8> for RefinementLevel {
    fn from(value: u8) -> Self {
        Self(value as i32)
    }
}

// u32 → Depth (infallible)
impl From<u32> for RefinementLevel {
    fn from(value: u32) -> Self {
        Self(value as i32)
    }
}

// Depth → i32
impl From<RefinementLevel> for i32 {
    fn from(d: RefinementLevel) -> Self {
        d.0
    }
}

// Depth → u8 (fallible)
impl TryFrom<RefinementLevel> for u8 {
    type Error = DggrsError;

    fn try_from(d: RefinementLevel) -> Result<Self, Self::Error> {
        u8::try_from(d.0).map_err(|_| DggrsError::RefinementLevelTooHigh(d))
    }
}

// Display for Depth
impl fmt::Display for RefinementLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativeDepth(i32);

impl RelativeDepth {
    pub fn new(value: i32) -> Result<Self, DggrsError> {
        if value < 0 {
            Err(DggrsError::RelativeDepthBelowZero(value))
        } else {
            Ok(Self(value))
        }
    }

    pub fn get(self) -> i32 {
        self.0
    }

    pub const fn new_const(val: i32) -> Self {
        Self(val)
    }
}

// i32 → RelativeDepth (fallible)
impl TryFrom<i32> for RelativeDepth {
    type Error = DggrsError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        RelativeDepth::new(value)
    }
}

// u8 → RelativeDepth (infallible)
impl From<u8> for RelativeDepth {
    fn from(value: u8) -> Self {
        Self(value as i32)
    }
}

// u32 → RelativeDepth (infallible)
impl From<u32> for RelativeDepth {
    fn from(value: u32) -> Self {
        Self(value as i32)
    }
}

// RelativeDepth → i32
impl From<RelativeDepth> for i32 {
    fn from(rd: RelativeDepth) -> Self {
        rd.0
    }
}

// RelativeDepth → u8 (fallible)
impl TryFrom<RelativeDepth> for u8 {
    type Error = DggrsError;

    fn try_from(rd: RelativeDepth) -> Result<Self, Self::Error> {
        u8::try_from(rd.0).map_err(|_| DggrsError::RelativeDepthTooLarge(rd))
    }
}

// Display for RelativeDepth
impl fmt::Display for RelativeDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
