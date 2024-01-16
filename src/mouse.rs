use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::camera::MainCamera;
use crate::grids::common::GridCoords;

pub struct MousePlugin;
impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseInfo::default());
        app.add_systems(PreUpdate, update_mouse_info_system);

    }
}

#[derive(Resource, Default)]
pub struct MouseInfo {
    pub screen_position: Vec2,
    pub world_position: Vec2,
    pub grid_coords: GridCoords, // Not guaranteed to be in bounds
    pub is_over_ui: bool,
}

pub fn update_mouse_info_system(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ui_nodes: Query<&Interaction, With<Node>>,
    mut mouse_info: ResMut<MouseInfo>,
) {
    let window = window.single();
    let (camera, camera_transform) = camera.single();

    if let Some(screen_position) = window.cursor_position() {
        let world_position = camera.viewport_to_world_2d(camera_transform, screen_position).unwrap();
        let grid_coords = GridCoords::from_world_vec2(world_position);
        // Update mouse info
        mouse_info.screen_position = screen_position;
        mouse_info.world_position = world_position;
        mouse_info.grid_coords = grid_coords;
    }
    if !ui_nodes.is_empty() {
        mouse_info.is_over_ui = ui_nodes.iter().any(|interaction| !matches!(interaction, Interaction::None));
    }
}