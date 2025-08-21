#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct TowerRangeCell {
    signature: u32,
    selected: u32,
    preview: u32,
}

struct UniformData {
    grid_width: u32,
    grid_height: u32,
}

@group(2) @binding(0) var<storage, read> cells: array<TowerRangeCell>;
@group(2) @binding(1) var<uniform> uniforms: UniformData;

// Rendering constants
const blockSize: f32 = 16.0; // Size of each block in pixels
const outlineThickness: f32 = 2.0; // Size of the outline in pixels
const outlineRatio: f32 = outlineThickness / blockSize; // Outline thickness relative to cell size

const BASE_COLOR: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0); // Transparent
const OUTLINE_COLOR: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.2); // White, dim
const SELECTED_FILL_COLOR: vec4<f32> = vec4<f32>(0.0, 0.9, 1.0, 12.0/255.0); // Cyan fill
const PREVIEW_FILL_COLOR: vec4<f32> = vec4<f32>(0.2, 1.0, 0.2, 15.0/255.0);  // Green fill
const SELECTED_OUTLINE_COLOR: vec4<f32> = vec4<f32>(0.0, 1.0, 1.0, 0.9); // Cyan bright outline
const PREVIEW_OUTLINE_COLOR: vec4<f32> = vec4<f32>(0.2, 1.0, 0.2, 0.9);  // Green bright outline

fn get_cell_data(uv: vec2<f32>) -> TowerRangeCell {
    // Clamp UV coordinates to valid range to prevent bleeding at edges
    let clamped_uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(0.9999));
    let grid_pos = vec2<u32>(clamped_uv * vec2<f32>(f32(uniforms.grid_width), f32(uniforms.grid_height)));
    let index = grid_pos.y * uniforms.grid_width + grid_pos.x;

    // Additional safety check: ensure grid coordinates are within bounds
    if (grid_pos.x >= uniforms.grid_width || grid_pos.y >= uniforms.grid_height || index >= arrayLength(&cells)) {
        return TowerRangeCell(0u, 0u, 0u); // Return empty cell if out of bounds
    }

    return cells[index];
}

struct EdgeInfo {
    signature_boundary: bool,
    selected_boundary: bool,
    preview_boundary: bool,
}

fn edge_info_between(a: TowerRangeCell, b: TowerRangeCell) -> EdgeInfo {
    return EdgeInfo(
        a.signature != b.signature,
        a.selected != b.selected,
        a.preview != b.preview,
    );
}

fn check_neighbor(curr: TowerRangeCell, neighborUV: vec2<f32>) -> EdgeInfo {
    if (neighborUV.x >= 0.0 && neighborUV.x < 1.0 && neighborUV.y >= 0.0 && neighborUV.y < 1.0) {
        return edge_info_between(curr, get_cell_data(neighborUV));
    }
    return EdgeInfo(false, false, false);
}

fn analyse_edge_boundaries(uv: vec2<f32>, blockPosition: vec2<f32>) -> EdgeInfo {
    let texDim = vec2<u32>(uniforms.grid_width, uniforms.grid_height);
    let stepSize = vec2<f32>(1.0 / f32(texDim.x), 1.0 / f32(texDim.y));
    let current = get_cell_data(uv);

    if (blockPosition.x <= outlineRatio) {
        // Check left neighbor
        let edge_details = check_neighbor(current, uv + vec2<f32>(-stepSize.x, 0.0));
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) { return edge_details; }
    } else if (blockPosition.x >= (1.0 - outlineRatio)) {
        // Check right neighbor
        let edge_details = check_neighbor(current, uv + vec2<f32>(stepSize.x, 0.0));
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) { return edge_details; }
    }

    if (blockPosition.y <= outlineRatio) {
        // Check bottom neighbor
        let edge_details = check_neighbor(current, uv + vec2<f32>(0.0, -stepSize.y));
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) { return edge_details; }
    } else if (blockPosition.y >= (1.0 - outlineRatio)) {
        // Check top neighbor
        let edge_details = check_neighbor(current, uv + vec2<f32>(0.0, stepSize.y));
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) { return edge_details; }
    }

    // Corner pixels: if both x and y are within the outline band, also check diagonal neighbors.
    // This fills inner/outer corners where axis-only checks miss a pixel.
    if (blockPosition.x <= outlineRatio && blockPosition.y <= outlineRatio) {
        // Bottom-left corner -> sample bottom-left diagonal
        let edge_details = check_neighbor(current, uv + vec2<f32>(-stepSize.x, -stepSize.y));
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) { return edge_details; }
    } else if (blockPosition.x <= outlineRatio && blockPosition.y >= (1.0 - outlineRatio)) {
        // Top-left corner -> sample top-left diagonal
        let edge_details = check_neighbor(current, uv + vec2<f32>(-stepSize.x, stepSize.y));
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) { return edge_details; }
    } else if (blockPosition.x >= (1.0 - outlineRatio) && blockPosition.y <= outlineRatio) {
        // Bottom-right corner -> sample bottom-right diagonal
        let edge_details = check_neighbor(current, uv + vec2<f32>(stepSize.x, -stepSize.y));
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) { return edge_details; }
    } else if (blockPosition.x >= (1.0 - outlineRatio) && blockPosition.y >= (1.0 - outlineRatio)) {
        // Top-right corner -> sample top-right diagonal
        let edge_details = check_neighbor(current, uv + vec2<f32>(stepSize.x, stepSize.y));
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) { return edge_details; }
    }

    return EdgeInfo(false, false, false);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    // Position within the block (cell)
    let blockPosition = fract(uv * vec2<f32>(f32(uniforms.grid_width), f32(uniforms.grid_height)));

    // Check if we're at the very edge of a block
    let atEdge = blockPosition.x <= outlineRatio || blockPosition.x >= (1.0 - outlineRatio) ||
                 blockPosition.y <= outlineRatio || blockPosition.y >= (1.0 - outlineRatio);

    // Get cell data from buffer
    let cell = get_cell_data(uv);

    // Base color: transparent
    var color = BASE_COLOR;

    // Fills
    if (cell.selected != 0u) {
        color = SELECTED_FILL_COLOR;
    }
    if (cell.preview != 0u) {
        // Preview overrides selected tint
        color = PREVIEW_FILL_COLOR;
    }

    // Outlines on boundaries
    if (atEdge) {
        let edge_details = analyse_edge_boundaries(uv, blockPosition);
        if (edge_details.signature_boundary || edge_details.selected_boundary || edge_details.preview_boundary) {
            var outline = OUTLINE_COLOR;
            if (edge_details.preview_boundary) {
                outline = PREVIEW_OUTLINE_COLOR;
            } else if (edge_details.selected_boundary) {
                outline = SELECTED_OUTLINE_COLOR;
            }
            color = outline;
        }
    }

    return color;
}
