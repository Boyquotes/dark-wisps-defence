#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct UniformData {
    highligh_enabled: u32, // 0 or 1
}

@group(2) @binding(0) var heatmap: texture_2d<f32>;
@group(2) @binding(1) var heatmap_sampler: sampler;
@group(2) @binding(2) var<uniform> uniforms: UniformData;

// Rendering constants
const blockSize: f32 = 16.; // Size of each block in pixels
const outlineThickness: f32 = 2.; // Size of the outline in pixels
const BASE_COLOR: vec4<f32> = vec4<f32>(1., 1., 1., 0.); // Transparent
const HAS_POWER_COLOR: vec4<f32> = vec4<f32>(1., 1., 0., 0.5); // Yellow
const NO_POWER_COLOR: vec4<f32> = vec4<f32>(1., 0.2, 0., 0.5); // Orange

// Thresholds for pixel decoding
const SUPPLY_THRESHOLD: f32 = 0.0;
const HIGHLIGHT_THRESHOLD: f32 = 5.0 / 255.0; // This value represents dimmed state. Being above it means it's highlighted.
const POWER_THRESHOLD: f32 = 0.5; // Red channel threshold

struct SupplyDetails {
    has_supply: bool,
    has_power: bool,
    is_highlighted: bool,
}
struct EdgeDetails {
    is_supply_boundary: bool,
    is_power_boundary: bool,
    is_highlight_boundary: bool,
}

fn supply_details(uv: vec2<f32>) -> SupplyDetails {
    let pixel = textureSample(heatmap, heatmap_sampler, uv);
    return SupplyDetails(
        pixel.a > SUPPLY_THRESHOLD,    // has_supply
        pixel.r < POWER_THRESHOLD,     // has_power (red=0 means power)
        pixel.a > HIGHLIGHT_THRESHOLD  // is_highlighted
    );
}
fn egde_details_from_supply_details(supply_details1: SupplyDetails, supply_details2: SupplyDetails) -> EdgeDetails {
    return EdgeDetails(
        supply_details1.has_supply != supply_details2.has_supply,
        supply_details1.has_power != supply_details2.has_power,
        supply_details1.is_highlighted != supply_details2.is_highlighted
    );
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
            if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) {
                return edge_details;
            }
        }
    } else if (blockPosition.x > ((blockSize - outlineThickness) / blockSize)) {
        // Check right neighbor
        let neighborUV = uv + vec2<f32>(stepSize.x, 0.0);
        if (neighborUV.x < 1.0) {
            let edge_details = egde_details_from_supply_details(supplyDetailsCurrent, supply_details(neighborUV));
            if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) {
                return edge_details;
            }
        }
    }

    if (blockPosition.y < (outlineThickness / blockSize)) {
        // Check bottom neighbor
        let neighborUV = uv + vec2<f32>(0.0, -stepSize.y);
        if (neighborUV.y >= 0.0) {
            let edge_details = egde_details_from_supply_details(supplyDetailsCurrent, supply_details(neighborUV));
            if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) {
                return edge_details;
            }
        }
    } else if (blockPosition.y > ((blockSize - outlineThickness) / blockSize)) {
        // Check top neighbor
        let neighborUV = uv + vec2<f32>(0.0, stepSize.y);
        if (neighborUV.y < 1.0) {
            let edge_details = egde_details_from_supply_details(supplyDetailsCurrent, supply_details(neighborUV));
            if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) {
                return edge_details;
            }
        }
    }

    return EdgeDetails(false, false, false);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    // Calculate position within the block (0.0 to blockSize)
    let blockPosition = fract(uv * f32(textureDimensions(heatmap, 0).x));

    // Check if we're at the very edge of a block
    let atEdge = blockPosition.x <= (outlineThickness / blockSize) || blockPosition.x >= ((blockSize - outlineThickness) / blockSize) ||
                 blockPosition.y <= (outlineThickness / blockSize) || blockPosition.y >= ((blockSize - outlineThickness) / blockSize);

    // First set general color for the block
    // If there's no supply, use the base color, otherwise check supply vs power
    var base_color = BASE_COLOR;
    let cell_data = textureSample(heatmap, heatmap_sampler, uv);
    let no_power_indicator = cell_data.r;
    if cell_data.a > 0.0 {
        base_color = select(HAS_POWER_COLOR, NO_POWER_COLOR, no_power_indicator > 0.0);
        base_color.a = cell_data.a;
    }

    // If we're at an edge and it's a boundary, draw the outline. Outline spills to other blocks so make sure you select proper color for it.
    let edge_details = is_edge_boundary(uv, blockPosition);
    if (atEdge && (edge_details.is_supply_boundary || edge_details.is_highlight_boundary)) {
        base_color = select(HAS_POWER_COLOR, NO_POWER_COLOR, edge_details.is_power_boundary);
        if (uniforms.highligh_enabled == 0 || edge_details.is_highlight_boundary) {
            base_color.a = 0.9; // Outline color
        } else if (edge_details.is_supply_boundary) {
            base_color.a = 0.2; // Outline color
        }
    }

    return base_color;
}