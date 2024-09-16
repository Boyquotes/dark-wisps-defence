#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var heatmap: texture_2d<f32>;
@group(2) @binding(1) var heatmap_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    let blockPosition = fract(uv * f32(textureDimensions(heatmap, 0).x) );

    // Sample the base color from the heatmap as usual
    var baseColor = textureSample(heatmap, heatmap_sampler, uv);

    // Visualize the blockPosition to debug
    // This will color the edges of each 16x16 block
    if (blockPosition.x < (1.0 / blockSize) || blockPosition.x > ((blockSize - 1.0) / blockSize) ||
        blockPosition.y < (1.0 / blockSize) || blockPosition.y > ((blockSize - 1.0) / blockSize)) {
        baseColor = vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red color for edges
    }

    return baseColor;
}