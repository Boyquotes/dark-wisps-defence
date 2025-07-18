mod buildings;
mod data_loader;
mod effects;
mod inventory;
mod map_editor;
mod map_loader;
mod map_objects;
mod overlays;
mod prelude;
mod projectiles;
mod ui;
mod units;
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
            inventory::InventoryPlugin,
            map_objects::MapObjectsPlugin,
            overlays::OverlaysPlugin,
            projectiles::ProjectilesPlugin,
            ui::UiPlugin,
            units::UnitsPlugin,
            wisps::WispsPlugin,
        ))
        .add_plugins((
            lib_grid::grids::GridsPlugin,
            lib_core::LibCorePlugin,
        ))
        .add_plugins((
            data_loader::DataLoaderPlugin,
            map_editor::MapEditorPlugin,
        ))
        .add_systems(Startup, |mut commands: Commands| commands.queue(map_loader::LoadMapCommand("test_map".to_string())))
        .run();
}