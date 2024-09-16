#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Wave parameters used in the ripple effect:
//
// current_radius (f32):
// - Normalized radius of the ripple's leading edge (range: 0.0 to 1.0).
// - Calculated as: current_radius_world / max_radius_world.
// - Controls how far the ripple has expanded.
//
// wave_width (f32):
// - Normalized width of the ripple behind the leading edge (range: 0.0 to 1.0).
// - Calculated as: wave_width_world / max_radius_world.
// - Determines the thickness of the ripple.
//
// wave_exponent (f32):
// - Controls the sharpness of the ripple's trailing edge fade.
// - Higher values result in a sharper edge.
// - Common values:
//   - 1.0: Linear fade.
//   - 2.0: Quadratic fade.
//   - 5.0+: Very sharp fade.
struct RippleData {
    current_radius: f32,
    wave_width: f32,
    wave_exponent: f32,
};

@group(2) @binding(0)
var<uniform> uniforms: RippleData;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(mesh.uv, center);

    let current_radius = uniforms.current_radius;
    let wave_width = mix(uniforms.wave_width, 0.01, uniforms.current_radius * 2.); // Shorten the ripple to the edge as it expands
    let wave_exponent = uniforms.wave_exponent;
    var alpha = 0.;

    // Apply the ripple effect across the entire mesh
    if (dist >= (current_radius - wave_width) && (dist <= current_radius)) {
        let diff = (current_radius - dist) / wave_width; 
        alpha = pow(1.0 - diff, uniforms.wave_exponent);
    }
    // Output color with alpha for blending
    return vec4<f32>(1.0, 1.0, 1.0, alpha);
}