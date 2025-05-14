#[derive(Debug)]
pub struct PositionGeo {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

impl Position2D {
    pub fn from_tuple(t: (u8, u8)) -> Position2D {
        Position2D { x: f64::from(t.0), y: f64::from(t.1) }
    }
    pub fn mid(a: Position2D, b: Position2D) -> Position2D {
        Position2D {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        }
    }
}
