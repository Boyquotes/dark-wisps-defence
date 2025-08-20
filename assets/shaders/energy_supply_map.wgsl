#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct EnergySupplyCell {
    has_supply: u32,
    has_power: u32,
    highlight_level: u32,
}

struct UniformData {
    grid_width: u32,
    grid_height: u32,
}

@group(2) @binding(0) var<storage, read> energy_cells: array<EnergySupplyCell>;
@group(2) @binding(1) var<uniform> uniforms: UniformData;

// Rendering constants
const blockSize: f32 = 16.; // Size of each block in pixels
const outlineThickness: f32 = 2.; // Size of the outline in pixels
const outlineRatio: f32 = outlineThickness / blockSize; // Outline thickness relative to cell size
const BASE_COLOR: vec4<f32> = vec4<f32>(1., 1., 1., 0.); // Transparent
const HAS_POWER_COLOR: vec4<f32> = vec4<f32>(1., 1., 0., 0.5); // Yellow
const NO_POWER_COLOR: vec4<f32> = vec4<f32>(1., 0.2, 0., 0.5); // Orange

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

fn get_cell_data(uv: vec2<f32>) -> EnergySupplyCell {
    // Clamp UV coordinates to valid range to prevent bleeding at edges
    let clamped_uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(0.9999));
    let grid_pos = vec2<u32>(clamped_uv * vec2<f32>(f32(uniforms.grid_width), f32(uniforms.grid_height)));
    let index = grid_pos.y * uniforms.grid_width + grid_pos.x;
    
    // Additional safety check: ensure grid coordinates are within bounds
    if (grid_pos.x >= uniforms.grid_width || grid_pos.y >= uniforms.grid_height ||  index >= arrayLength(&energy_cells)) {
        return EnergySupplyCell(0u, 0u, 0u); // Return empty cell if out of bounds
    }
    
    return energy_cells[index];
}

fn supply_details(uv: vec2<f32>) -> SupplyDetails {
    let cell = get_cell_data(uv);
    return SupplyDetails(
        cell.has_supply != 0u,      // has_supply
        cell.has_power != 0u,       // has_power
        cell.highlight_level == 2u  // is_highlighted (2 = Highlighted)
    );
}
fn edge_details_from_supply_details(supply_details1: SupplyDetails, supply_details2: SupplyDetails) -> EdgeDetails {
    return EdgeDetails(
        supply_details1.has_supply != supply_details2.has_supply,
        supply_details1.has_power != supply_details2.has_power,
        supply_details1.is_highlighted != supply_details2.is_highlighted
    );
}

fn check_neighbor(supplyDetailsCurrent: SupplyDetails, neighborUV: vec2<f32>) -> EdgeDetails {
    if (neighborUV.x >= 0.0 && neighborUV.x < 1.0 && neighborUV.y >= 0.0 && neighborUV.y < 1.0) {
        return edge_details_from_supply_details(supplyDetailsCurrent, supply_details(neighborUV));
    }
    return EdgeDetails(false, false, false);
}

fn analyse_edge_boundaries(uv: vec2<f32>, blockPosition: vec2<f32>) -> EdgeDetails {
    let texDim = vec2<u32>(uniforms.grid_width, uniforms.grid_height);
    let stepSize = vec2<f32>(1.0 / f32(texDim.x), 1.0 / f32(texDim.y));
    let supplyDetailsCurrent = supply_details(uv);

    if (blockPosition.x <= outlineRatio) {
        // Check left neighbor
        let edge_details = check_neighbor(supplyDetailsCurrent, uv + vec2<f32>(-stepSize.x, 0.0));
        if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) { return edge_details; }
    } else if (blockPosition.x >= (1.0 - outlineRatio)) {
        // Check right neighbor
        let edge_details = check_neighbor(supplyDetailsCurrent, uv + vec2<f32>(stepSize.x, 0.0));
        if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) { return edge_details; }
    }

    if (blockPosition.y <= outlineRatio) {
        // Check bottom neighbor
        let edge_details = check_neighbor(supplyDetailsCurrent, uv + vec2<f32>(0.0, -stepSize.y));
        if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) { return edge_details; }
    } else if (blockPosition.y >= (1.0 - outlineRatio)) {
        // Check top neighbor
        let edge_details = check_neighbor(supplyDetailsCurrent, uv + vec2<f32>(0.0, stepSize.y));
        if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) { return edge_details; }
    }

    // Corner pixels: if both x and y are within the outline band, also check diagonal neighbors.
    // This fills inner/outer corners where axis-only checks miss a pixel.
    if (blockPosition.x <= outlineRatio && blockPosition.y <= outlineRatio) {
        // Bottom-left corner -> sample bottom-left diagonal
        let edge_details = check_neighbor(supplyDetailsCurrent, uv + vec2<f32>(-stepSize.x, -stepSize.y));
        if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) { return edge_details; }
    } else if (blockPosition.x <= outlineRatio && blockPosition.y >= (1.0 - outlineRatio)) {
        // Top-left corner -> sample top-left diagonal
        let edge_details = check_neighbor(supplyDetailsCurrent, uv + vec2<f32>(-stepSize.x, stepSize.y));
        if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) { return edge_details; }
    } else if (blockPosition.x >= (1.0 - outlineRatio) && blockPosition.y <= outlineRatio) {
        // Bottom-right corner -> sample bottom-right diagonal
        let edge_details = check_neighbor(supplyDetailsCurrent, uv + vec2<f32>(stepSize.x, -stepSize.y));
        if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) { return edge_details; }
    } else if (blockPosition.x >= (1.0 - outlineRatio) && blockPosition.y >= (1.0 - outlineRatio)) {
        // Top-right corner -> sample top-right diagonal
        let edge_details = check_neighbor(supplyDetailsCurrent, uv + vec2<f32>(stepSize.x, stepSize.y));
        if (edge_details.is_supply_boundary || edge_details.is_highlight_boundary) { return edge_details; }
    }

    return EdgeDetails(false, false, false);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    // Calculate position within the block (0.0 to blockSize)
    let blockPosition = fract(uv * vec2<f32>(f32(uniforms.grid_width), f32(uniforms.grid_height)));

    // Check if we're at the very edge of a block
    let atEdge = blockPosition.x <= outlineRatio || blockPosition.x >= (1.0 - outlineRatio) ||
                 blockPosition.y <= outlineRatio || blockPosition.y >= (1.0 - outlineRatio);

    // Get cell data from buffer
    let cell = get_cell_data(uv);
    let supply_detail = supply_details(uv);
    
    // First set general color for the block
    // If there's no supply, use the base color, otherwise check supply vs power
    var base_color = BASE_COLOR;
    if supply_detail.has_supply {
        base_color = select(NO_POWER_COLOR, HAS_POWER_COLOR, supply_detail.has_power);
        
        // Set proper alpha based on highlight level
        // 0 = None (transparent), 1 = Dimmed (5/255), 2 = Highlighted (15/255)
        if cell.highlight_level == 0u {
            base_color.a = 0.0; // Transparent (no highlight)
        } else if cell.highlight_level == 1u {
            base_color.a = 5.0 / 255.0; // Dimmed highlight
        } else {
            base_color.a = 15.0 / 255.0; // Normal/Highlighted
        }
    }

    // If we're at an edge and it's a boundary, draw the outline
    let edge_details = analyse_edge_boundaries(uv, blockPosition);
    if (atEdge && (edge_details.is_supply_boundary || edge_details.is_highlight_boundary)) {
        // For power boundaries, always use the powered color (yellow) for the outline
        // For non-power boundaries, use the current cell's color
        if (edge_details.is_power_boundary) {
            base_color = HAS_POWER_COLOR; // Always yellow for power boundaries
        } else {
            base_color = select(NO_POWER_COLOR, HAS_POWER_COLOR, supply_detail.has_power);
        }
        
        // Set outline alpha - simplified logic using cell highlight levels
        if (edge_details.is_highlight_boundary) {
            base_color.a = 0.9; // Bright outline for highlight boundaries
        } else if (edge_details.is_supply_boundary) {
            base_color.a = 0.2; // Dimmed outline for supply boundaries
        }
    }

    return base_color;
}