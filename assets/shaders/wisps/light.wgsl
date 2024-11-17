#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct UniformData {
    radiance_speed: f32,
};

@group(2) @binding(4)
var<uniform> uniforms: UniformData;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(mesh.uv, center);

    // Adjust the intensity using an exponential decay
    let core_intensity = clamp(exp(-dist * dist * 16.0), 0.0, 1.0);

    // Radiance effect parameters
    let radiance_width = 0.55;      // Width of the radiance band
    let radiance_offset = fract(globals.time * uniforms.radiance_speed);

    // Calculate the radiance intensity
    let radiance = smoothstep(radiance_offset, radiance_offset + radiance_width, dist);

    // Combine core and radiance intensities
    let final_intensity = core_intensity + (1.0 - radiance) * 0.5;

    // Clamp the final intensity to [0,1]
    let intensity = clamp(final_intensity, 0.5, 1.0);

    // Set the color based on the intensity
    let color = vec3<f32>(intensity);

    // Adjust opacity to fade out towards the edges
    let alpha = clamp(1.0 - dist * 2.0, 0.0, 1.0);

    // Output the final color with opacity dimishing outwards
    return vec4<f32>(color, alpha);
}