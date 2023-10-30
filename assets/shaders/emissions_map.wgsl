#import bevy_pbr::mesh_vertex_output MeshVertexOutput

@group(1) @binding(0) var heatmap: texture_2d<f32>;
@group(1) @binding(1) var heatmap_sampler: sampler;

@fragment
fn fragment(mesh: MeshVertexOutput) -> @location(0) vec4<f32> {
//    let block_size = 16.0;
//    let scaled_uv = vec2<f32>(
//        floor(mesh.uv.x * 100.0 / block_size),
//        floor(mesh.uv.y * 100.0 / block_size)
//    ) * block_size / 100.0;
    return textureSample(heatmap, heatmap_sampler, mesh.uv);
}