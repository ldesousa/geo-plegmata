use super::vector_3d::Vector3D;

#[derive(Debug)]
pub struct Quaternion {
    // Assuming a unit quaternion with x, y, z, w
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Quaternion {
    pub fn yaw_pitch(yaw: f64, pitch: f64) -> Self {
        let (sy, cy) = ((yaw * 0.5).sin(), (yaw * 0.5).cos());
        let (sp, cp) = ((pitch * 0.5).sin(), (pitch * 0.5).cos());

        Self {
            w: cy * cp,
            x: sy * sp,
            y: sy * cp,
            z: cy * sp,
        }
    }
    // s′=q⋅s⋅q⁻1
    pub fn rotate_vector(&self, v: Vector3D) -> Vector3D {
        let q = Vector3D {
            x: self.x,
            y: self.y,
            z: self.z,
        };
        let w = self.w;
        let a = w * w - q.dot(q);
        let dot_qv = q.dot(v);
        let cross = v.cross(q);

        Vector3D {
            x: 2.0 * dot_qv * q.x + a * v.x + 2.0 * w * cross.x,
            y: 2.0 * dot_qv * q.y + a * v.y + 2.0 * w * cross.y,
            z: 2.0 * dot_qv * q.z + a * v.z + 2.0 * w * cross.z,
        }
    }
}