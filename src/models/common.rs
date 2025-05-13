#[derive(Debug)]
pub struct PositionGeo {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

impl Position2D {
    pub fn mid(a: Self, b: Self) -> Self {
        Self {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        }
    }
}
