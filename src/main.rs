mod buildings;
mod camera;
mod common_components;
mod common_systems;
mod common;
mod data_loader;
mod effects;
mod grids;
mod inventory;
mod map_editor;
mod map_loader;
mod map_objects;
mod mouse;
mod overlays;
mod prelude;
mod projectiles;
mod search;
mod ui;
mod units;
mod utils;
mod wisps;

use crate::prelude::*;
use crate::grids::obstacles::ObstacleGrid;
use crate::map_editor::MapInfo;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(30, 31, 34)))
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin{ primary_window: Some(Window { present_mode: bevy::window::PresentMode::AutoNoVsync, ..default()}), ..default() }),
            buildings::BuildingsPlugin,
            effects::EffectsPlugin,
            grids::GridsPlugin,
            inventory::InventoryPlugin,
            map_objects::MapObjectsPlugin,
            overlays::OverlaysPlugin,
            projectiles::ProjectilesPlugin,
            ui::UiPlugin,
            units::UnitsPlugin,
            wisps::WispsPlugin,
        ))
        .add_plugins((
            camera::CameraPlugin,
            common::CommonPlugin,
            common_systems::CommonSystemsPlugin,
            data_loader::DataLoaderPlugin,
            map_editor::MapEditorPlugin,
            mouse::MousePlugin,
        ))
        .add_systems(Startup, generate_default_map)
        .run();
}

pub fn generate_default_map(
    mut commands: Commands,
    mut obstacles_grid: ResMut<ObstacleGrid>,
    mut map_info: ResMut<MapInfo>,
    almanach: Res<Almanach>,
) {
    let map = map_loader::load_map("test_map");
    map_info.name = "test_map".to_string();
    map_info.grid_width = map.width;
    map_info.grid_height = map.height;
    map_info.world_width = map.width as f32 * CELL_SIZE;
    map_info.world_height = map.height as f32 * CELL_SIZE;
    map_loader::apply_map(
        map,
        &mut commands, 
        &mut obstacles_grid,
        &almanach,
    );
}