
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

pub fn angle_difference(angle1: f32, angle2: f32) -> f32 {
    let diff = (angle1 - angle2) % (2.0 * std::f32::consts::PI);
    if diff < -std::f32::consts::PI {
        diff + 2.0 * std::f32::consts::PI
    } else {
        diff
    }
}