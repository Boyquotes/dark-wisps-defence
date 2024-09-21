#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct UniformData {
    amplitude: f32,
    frequency: f32,
    speed: f32, // animation speed
    sinus_direction: f32, // -1 or 1
    cosinus_direction: f32, // -1 or 1
};

@group(2) @binding(4)
var<uniform> uniforms: UniformData;

@group(2) @binding(0) var wisp_tex1: texture_2d<f32>;
@group(2) @binding(1) var wisp_tex1_sampler: sampler;
@group(2) @binding(2) var wisp_tex2: texture_2d<f32>;

const circle_radius: f32 = 0.64;
const edge_softness: f32 = 0.35;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(mesh.uv, center);
    let amplitude = uniforms.amplitude;  // 0.2 - 0.45, 0.4311
    let frequency =  uniforms.frequency; // 15 - 22, 15.5
    let speed = uniforms.speed; // 3-7, 5.

    // Calculate distortion based on UV coordinates and time
    let offset = vec2<f32>(
        amplitude * sin(mesh.uv.y * frequency + globals.time * speed * uniforms.sinus_direction), 
        amplitude * cos(mesh.uv.x * frequency + globals.time * speed * uniforms.cosinus_direction) 
    );


    // Apply distortion to UV coordinates
    let distorted_uv = mesh.uv + offset;

    // Sample the texture with distorted UV
    var color = vec4<f32>(1.0 - dist * 2., 0., 0., 1.0 - dist * 2.);
    if dist > 0.25 {
        color = textureSample(wisp_tex1, wisp_tex1_sampler, distorted_uv) + textureSample(wisp_tex2, wisp_tex1_sampler, distorted_uv);
        color.r = 1.;
        color.g += 0.07;
        color.b += 0.07;
        let mask = smoothstep(circle_radius, circle_radius - edge_softness, dist);
        color.a *= mask;
        // Get rid of non-black parts
        if color.g > 0.2 {
            color.a = 0.0;
        }
    } else if dist > 0.20 {
        // Make a gap between the wisp core and the aura effect
        color.a = 0.;
    }
    
    return color;
}