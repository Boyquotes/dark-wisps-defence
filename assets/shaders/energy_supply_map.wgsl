#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var heatmap: texture_2d<f32>;
@group(2) @binding(1) var heatmap_sampler: sampler;

const blockSize: f32 = 16.0; // Size of each block in pixels
const outlineThickness: f32 = 2.; // Size of the outline in pixels

fn has_supply(uv: vec2<f32>) -> bool {
    let pixel = textureSample(heatmap, heatmap_sampler, uv);
    return pixel.a > 0.0;
}

fn is_edge_boundary(uv: vec2<f32>, blockPosition: vec2<f32>) -> bool {
    let texDim = textureDimensions(heatmap, 0);
    let stepSize = vec2<f32>(1.0 / f32(texDim.x), 1.0 / f32(texDim.y));
    let hasSupplyCurrent = has_supply(uv);

    if (blockPosition.x < (outlineThickness / blockSize)) {
        // Check left neighbor
        let neighborUV = uv + vec2<f32>(-stepSize.x, 0.0);
        if (neighborUV.x >= 0.0 && hasSupplyCurrent != has_supply(neighborUV)) {
            return true;
        }
    } else if (blockPosition.x > ((blockSize - outlineThickness) / blockSize)) {
        // Check right neighbor
        let neighborUV = uv + vec2<f32>(stepSize.x, 0.0);
        if (neighborUV.x < 1.0 && hasSupplyCurrent != has_supply(neighborUV)) {
            return true;
        }
    }

    if (blockPosition.y < (outlineThickness / blockSize)) {
        // Check bottom neighbor
        let neighborUV = uv + vec2<f32>(0.0, -stepSize.y);
        if (neighborUV.y >= 0.0 && hasSupplyCurrent != has_supply(neighborUV)) {
            return true;
        }
    } else if (blockPosition.y > ((blockSize - outlineThickness) / blockSize)) {
        // Check top neighbor
        let neighborUV = uv + vec2<f32>(0.0, stepSize.y);
        if (neighborUV.y < 1.0 && hasSupplyCurrent != has_supply(neighborUV)) {
            return true;
        }
    }

    return false;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    // Calculate position within the block (0.0 to blockSize)
    let blockPosition = fract(uv * f32(textureDimensions(heatmap, 0).x));

    // Check if we're at the very edge of a block
    let atEdge = blockPosition.x < (outlineThickness / blockSize) || blockPosition.x > ((blockSize - outlineThickness) / blockSize) ||
                 blockPosition.y < (outlineThickness / blockSize) || blockPosition.y > ((blockSize - outlineThickness) / blockSize);

    // Sample the base color from the heatmap
    var baseColor = textureSample(heatmap, heatmap_sampler, uv);

    // If we're at an edge and it's a boundary, draw the outline
    if (atEdge && is_edge_boundary(uv, blockPosition)) {
        baseColor.a = 0.9; // Outline color
    }

    return baseColor;
}