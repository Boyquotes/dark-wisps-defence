#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct UniformData {
    highligh_enabled: u32, // 0 or 1
}

@group(2) @binding(0) var heatmap: texture_2d<f32>;
@group(2) @binding(1) var heatmap_sampler: sampler;
@group(2) @binding(2) var<uniform> uniforms: UniformData;

const blockSize: f32 = 16.0; // Size of each block in pixels
const outlineThickness: f32 = 2.; // Size of the outline in pixels

struct SupplyDetails {
    has_supply: bool,
    is_highlighted: bool,
}
struct EdgeDetails {
    is_boundary: bool,
    is_highlight_boundary: bool,
}

fn supply_details(uv: vec2<f32>) -> SupplyDetails {
    let pixel = textureSample(heatmap, heatmap_sampler, uv);
    return SupplyDetails(pixel.a > 0.0, pixel.a > 0.1);
}
fn egde_details_from_supply_details(supply_details1: SupplyDetails, supply_details2: SupplyDetails) -> EdgeDetails {
    return EdgeDetails(supply_details1.has_supply != supply_details2.has_supply, supply_details1.is_highlighted != supply_details2.is_highlighted);
}

fn is_edge_boundary(uv: vec2<f32>, blockPosition: vec2<f32>) -> EdgeDetails {
    let texDim = textureDimensions(heatmap, 0);
    let stepSize = vec2<f32>(1.0 / f32(texDim.x), 1.0 / f32(texDim.y));
    let supplyDetailsCurrent = supply_details(uv);

    if (blockPosition.x < (outlineThickness / blockSize)) {
        // Check left neighbor
        let neighborUV = uv + vec2<f32>(-stepSize.x, 0.0);
        if (neighborUV.x >= 0.0) {
            let edge_details = egde_details_from_supply_details(supplyDetailsCurrent, supply_details(neighborUV));
            if (edge_details.is_boundary || edge_details.is_highlight_boundary) {
                return edge_details;
            }
        }
    } else if (blockPosition.x > ((blockSize - outlineThickness) / blockSize)) {
        // Check right neighbor
        let neighborUV = uv + vec2<f32>(stepSize.x, 0.0);
        if (neighborUV.x < 1.0) {
            let edge_details = egde_details_from_supply_details(supplyDetailsCurrent, supply_details(neighborUV));
            if (edge_details.is_boundary || edge_details.is_highlight_boundary) {
                return edge_details;
            }
        }
    }

    if (blockPosition.y < (outlineThickness / blockSize)) {
        // Check bottom neighbor
        let neighborUV = uv + vec2<f32>(0.0, -stepSize.y);
        if (neighborUV.y >= 0.0) {
            let edge_details = egde_details_from_supply_details(supplyDetailsCurrent, supply_details(neighborUV));
            if (edge_details.is_boundary || edge_details.is_highlight_boundary) {
                return edge_details;
            }
        }
    } else if (blockPosition.y > ((blockSize - outlineThickness) / blockSize)) {
        // Check top neighbor
        let neighborUV = uv + vec2<f32>(0.0, stepSize.y);
        if (neighborUV.y < 1.0) {
            let edge_details = egde_details_from_supply_details(supplyDetailsCurrent, supply_details(neighborUV));
            if (edge_details.is_boundary || edge_details.is_highlight_boundary) {
                return edge_details;
            }
        }
    }

    return EdgeDetails(false, false);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    // Calculate position within the block (0.0 to blockSize)
    let blockPosition = fract(uv * f32(textureDimensions(heatmap, 0).x));

    // Check if we're at the very edge of a block
    let atEdge = blockPosition.x <= (outlineThickness / blockSize) || blockPosition.x >= ((blockSize - outlineThickness) / blockSize) ||
                 blockPosition.y <= (outlineThickness / blockSize) || blockPosition.y >= ((blockSize - outlineThickness) / blockSize);

    // Sample the base color from the heatmap
    var baseColor = textureSample(heatmap, heatmap_sampler, uv);

    // If we're at an edge and it's a boundary, draw the outline
    let edge_details = is_edge_boundary(uv, blockPosition);
    if (atEdge && (edge_details.is_boundary || edge_details.is_highlight_boundary)) {
        if (uniforms.highligh_enabled == 0 || edge_details.is_highlight_boundary) {
            baseColor.a = 0.9; // Outline color
        } else if (edge_details.is_boundary) {
            baseColor.a = 0.2; // Outline color
        }
    }

    return baseColor;
}