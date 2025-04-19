use std::f64::consts::PI;

pub fn to_rad(value: f64) -> f64 {
    value * PI / 180.0
}

pub fn to_deg(value: f64) -> f64 {
    value * 180.0 / PI
}

pub fn pow(n: f64, e: i32) -> f64 {
    f64::powi(n, e)
}

pub fn cos(v: f64) -> f64 {
    f64::cos(v)
}

pub fn sin(v: f64) -> f64 {
    f64::sin(v)
}

pub fn tan(v: f64) -> f64 {
    f64::tan(v)
}

pub fn cot(v: f64) -> f64 {
    1.0 / tan(v)
}
