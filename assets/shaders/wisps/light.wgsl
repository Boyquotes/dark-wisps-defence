#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct UniformData {
    radiance_angle: f32, // 10.0 - 30.0
    radiance_radius: f32, // 5.0 - 15.0
};

@group(2) @binding(4)
var<uniform> uniforms: UniformData;

// Hash function to generate pseudo-random values
fn hash(p: vec2<f32>) -> f32 {
    let h: f32 = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

// Noise function using the hash function
fn noise(p: vec2<f32>) -> f32 {
    let i: vec2<f32> = floor(p);
    let f: vec2<f32> = fract(p);
    let a: f32 = hash(i);
    let b: f32 = hash(i + vec2<f32>(1.0, 0.0));
    let c: f32 = hash(i + vec2<f32>(0.0, 1.0));
    let d: f32 = hash(i + vec2<f32>(1.0, 1.0));
    let u: vec2<f32> = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(mesh.uv, center);

    // Center the UV coordinates around (0.0, 0.0)
    let centered_uv: vec2<f32> = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);

    // Define the small core of the orb
    let core_radius: f32 = 0.3;
    let core_intensity: f32 = smoothstep(core_radius, 0.0, dist);

    // Generate crackling energy bolts using noise
    let angle: f32 = atan2(centered_uv.y, centered_uv.x);
    let radius: f32 = dist;
    let bolt_noise: f32 = noise(vec2<f32>(angle * uniforms.radiance_angle, globals.time * 20.0 - radius * uniforms.radiance_radius));

    // Create bolts by thresholding the noise
    let bolt_intensity: f32 = smoothstep(0.3, 0.5, bolt_noise);

    // Combine core intensity and bolt intensity
    let core_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0) * core_intensity;
    let bolt_color: vec3<f32> = vec3<f32>(0.9, 0.9, 0.9) * bolt_intensity;

    // Combine the colors
    let color: vec3<f32> = core_color + bolt_color;

    // Define the maximum radius of the orb
    let max_radius = 0.8;

    // Calculate exponential fade for alpha based on distance
    let edge_fade = exp(-pow((dist / max_radius), 2.0) * 10.0);

    var alpha = 1.;
    // Apply the exponential fade to the alpha channel
    if color.r < 0.6 && color.g < 0.6 && color.b < 0.6 {
        alpha = 0.0;
    } else {
        alpha = clamp(edge_fade, 0.0, 1.0);
    }

    // Output the final color with full opacity
    return vec4<f32>(color, alpha);
}