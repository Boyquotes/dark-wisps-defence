use bevy::color::palettes::css::GRAY;
use crate::prelude::*;
use crate::grids::obstacles::ObstacleGrid;
use crate::ui::UiConfig;

pub fn show_hide_grid_system(mut ui_config: ResMut<UiConfig>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyG) {
        ui_config.show_grid = !ui_config.show_grid;
    }
}

pub fn draw_grid_system(grid: Res<ObstacleGrid>, ui_config: Res<UiConfig>, mut gizmos: Gizmos) {
    if !ui_config.show_grid { return; }

    let total_height = grid.height as f32 * CELL_SIZE;
    let total_width = grid.width as f32 * CELL_SIZE;

    // Horizontal lines
    for y in 0..=grid.height {
        let start = Vec2::new(0.0, y as f32 * CELL_SIZE);
        let end = Vec2::new(total_width, y as f32 * CELL_SIZE);
        gizmos.line_2d(start, end, GRAY);
    }

    // Vertical lines
    for x in 0..=grid.width {
        let start = Vec2::new(x as f32 * CELL_SIZE, 0.0);
        let end = Vec2::new(x as f32 * CELL_SIZE, total_height);
        gizmos.line_2d(start, end, GRAY);
    }
}
