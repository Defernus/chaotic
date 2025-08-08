/// Convert angle to a normalized value between 0 and 1
pub fn normalize_angle(angle: f64) -> f64 {
    let normalized = angle % (2.0 * std::f64::consts::PI);
    (if normalized < 0.0 {
        normalized + 2.0 * std::f64::consts::PI
    } else {
        normalized
    }) / (2.0 * std::f64::consts::PI)
}

pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
