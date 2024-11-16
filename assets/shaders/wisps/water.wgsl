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
    let amplitude = uniforms.amplitude;  // 0.06 - 0.14
    let frequency =  uniforms.frequency; // 2. - 2.6
    let speed = uniforms.speed; // 3.0 - 6.0

    //let amplitude = 0.24311;
    //let frequency =  2.;   // This gives worm like movement effect

    // Calculate distortion based on UV coordinates and time
    let offset = vec2<f32>(
        amplitude * sin(mesh.uv.y * frequency + globals.time * speed * uniforms.sinus_direction),
        amplitude * cos(mesh.uv.x * frequency + globals.time * speed * uniforms.cosinus_direction) 
    );


    // Apply distortion to UV coordinates
    let distorted_uv = mesh.uv + offset;

    // Sample the texture with distorted UV
    var color = vec4<f32>(0., 0., 1.0 - dist * 2., 1.0 - dist * 2.);
    color = textureSample(wisp_tex1, wisp_tex1_sampler, distorted_uv) + textureSample(wisp_tex2, wisp_tex1_sampler, distorted_uv);
    color.b = 1.;
    color.g += 0.07;
    color.r += 0.07;
    let mask = smoothstep(circle_radius, circle_radius - edge_softness, dist);
    color.a *= mask;
    // Get rid of non-black parts
    if color.g > 0.2 {
        color.a = 0.0;
    }
    
    return color;
}