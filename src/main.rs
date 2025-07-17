mod buildings;
mod camera;
mod common_components;
mod common_systems;
mod common;
mod data_loader;
mod effects;
mod inventory;
mod map_editor;
mod map_loader;
mod map_objects;
mod mouse;
mod overlays;
mod prelude;
mod projectiles;
mod ui;
mod units;
mod utils;
mod wisps;

use crate::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(30, 31, 34)))
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin{ primary_window: Some(Window { present_mode: bevy::window::PresentMode::AutoNoVsync, ..default()}), ..default() }),
            buildings::BuildingsPlugin,
            effects::EffectsPlugin,
            lib_grid::grids::GridsPlugin,
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
        .add_systems(Startup, |mut commands: Commands| commands.queue(map_loader::LoadMapCommand("test_map".to_string())))
        .run();
}