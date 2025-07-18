pub fn angle_difference(angle1: f32, angle2: f32) -> f32 {
    let mut diff = (angle1 - angle2) % (2.0 * std::f32::consts::PI);
    
    // Normalize to [-π, π] range for shortest rotation
    if diff > std::f32::consts::PI {
        diff -= 2.0 * std::f32::consts::PI;
    } else if diff < -std::f32::consts::PI {
        diff += 2.0 * std::f32::consts::PI;
    }
    
    diff
}