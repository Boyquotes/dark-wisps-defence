#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var heatmap: texture_2d<f32>;
@group(2) @binding(1) var heatmap_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(heatmap, heatmap_sampler, mesh.uv);
}