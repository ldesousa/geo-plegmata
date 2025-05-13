#[derive(Clone, Copy, Debug)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    pub fn normalize(self) -> Self {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Self {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    pub fn mid(a: Self, b: Self) -> Self {
        Self {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
            z: (a.z + b.z) / 2.0,
        }
    }
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}
