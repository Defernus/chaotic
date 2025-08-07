use nannou::glam::DVec2;

pub fn rotate_dvec2(vec: DVec2, angle: f64) -> DVec2 {
    let (sin, cos) = angle.sin_cos();
    DVec2::new(vec.x * cos - vec.y * sin, vec.x * sin + vec.y * cos)
}

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

pub fn map_index_to_f32(value: usize, len: usize, size: f32) -> f32 {
    value as f32 / len as f32 * size
}

pub fn map_f32_to_index(value: f32, len: usize, size: f32) -> usize {
    ((value.max(0.0) * len as f32 / size) as usize).min(len - 1)
}
