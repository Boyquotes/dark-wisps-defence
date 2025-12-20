mod buildings;
mod data_loader;
mod editor;
mod effects;
mod map_editor;
mod map_loader;
mod map_objects;
mod overlays;
mod objectives;
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
                // Warning: VSync is causing a lot of issues with mouse events processing
                .set(WindowPlugin{ primary_window: Some(Window { present_mode: bevy::window::PresentMode::AutoNoVsync, ..default()}), ..default() }),
            MeshPickingPlugin,
            buildings::BuildingsPlugin,
            effects::EffectsPlugin,
            map_objects::MapObjectsPlugin,
            objectives::ObjectivesPlugin,
            overlays::OverlaysPlugin,
            projectiles::ProjectilesPlugin,
            ui::UiPlugin,
            units::UnitsPlugin,
            wisps::WispsPlugin,
        ))
        .add_plugins((
            lib_grid::grids::GridsPlugin,
            lib_core::LibCorePlugin,
            lib_inventory::LibInventoryPlugin,
            lib_ui::LibUiPlugin,
        ))
        .add_plugins((
            data_loader::DataLoaderPlugin,
            editor::EditorPlugin,
            map_editor::MapEditorPlugin,
            map_loader::MapLoaderPlugin,
        ))
        // Warning: Bevy behaves wierdly when there are many Startup systems. If some plugins begin to not run at all, watch out for Startup systems at all places. Use Post/Pre Startup instead.
        .add_systems(PostStartup, |mut commands: Commands| commands.trigger(map_loader::LoadMapRequest("test_map".to_string())))
        .run();
}