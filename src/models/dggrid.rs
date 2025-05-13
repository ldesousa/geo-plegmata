use std::fmt;

#[derive(Debug)]
pub struct PointOld {
    pub lon: f64,
    pub lat: f64,
}
#[derive(Debug, Clone)]
pub struct CellID {
    pub id: String,
}

#[derive(Debug)]
pub struct CellCoord {
    pub lon: f64,
    pub lat: f64,
}
#[derive(Debug)]
pub struct CellGeom {
    pub geom: Vec<CellCoord>,
}
#[derive(Debug)]
pub struct Cell {
    pub id: CellID,
    pub geom: CellGeom,
}
#[derive(Debug)]
pub struct Cells {
    pub cells: Vec<Cell>,
}
#[derive(Debug)]
pub struct IdArray {
    pub id: Option<String>,
    pub arr: Option<Vec<String>>,
}

impl CellID {
    pub fn new(id: &str) -> Result<Self, String> {
        if (id.len() == 16 || id.len() == 18) && id.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(CellID { id: id.to_string() })
        } else {
            Err("ID must be exactly 16 or 18 alphanumeric characters.".to_string())
        }
    }
}

impl Default for CellID {
    fn default() -> Self {
        CellID {
            id: "0000000000000000".to_string(),
        } // Some valid default ID
    }
}

impl Default for CellGeom {
    fn default() -> Self {
        CellGeom { geom: Vec::new() }
    }
}

impl fmt::Display for CellID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl fmt::Display for CellCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Longitude: {:.6}, Latitude: {:.6}", self.lon, self.lat)
    }
}
