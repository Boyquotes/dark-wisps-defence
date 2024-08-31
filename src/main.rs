mod camera;
mod common;
mod buildings;
mod wisps;
mod common_components;
mod map_loader;
mod map_objects;
mod ui;
mod map_editor;
mod mouse;
mod utils;
mod projectiles;
mod grids;
mod search;
mod overlays;
mod inventory;
mod effects;
mod units;
mod common_systems;
mod prelude;

use crate::prelude::*;
use crate::grids::common::CELL_SIZE;
use crate::grids::emissions::EmissionsEnergyRecalculateAll;
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::ObstacleGrid;
use crate::inventory::objectives::ObjectivesCheckInactiveFlag;
use crate::map_editor::MapInfo;

fn main() {
    let mut grid = ObstacleGrid::new_empty();
    grid.resize_and_reset(10, 10);
    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(30, 31, 34)))
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            camera::CameraPlugin,
            grids::GridsPlugin,
            wisps::systems::WispsPlugin,
            ui::UiPlugin,
            map_editor::MapEditorPlugin,
            mouse::MousePlugin,
            map_objects::MapObjectsPlugin,
            buildings::BuildingsPlugin,
            projectiles::ProjectilesPlugin,
            overlays::OverlaysPlugin,
            inventory::InventoryPlugin,
            effects::EffectsPlugin,
            units::UnitsPlugin,
        ))
        .insert_resource(GameConfig{
            mode: GameMode::Editor,
        })
        .add_systems(Startup, generate_default_map)
        .add_systems(Update, common_systems::pulsate_sprites_system)
        .run();
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameMode {
    Game,
    Editor,
    Manu,
}

#[derive(Resource)]
pub struct GameConfig {
    pub mode: GameMode,
}

pub fn is_editor_mode(config: Res<GameConfig>) -> bool {
    config.mode == GameMode::Editor
}

pub fn is_game_mode(config: Res<GameConfig>) -> bool {
    true || config.mode == GameMode::Game
}

pub fn generate_default_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut objectives_check_inactive_flag: ResMut<ObjectivesCheckInactiveFlag>,
    mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut obstacles_grid: ResMut<ObstacleGrid>,
    mut map_info: ResMut<MapInfo>,
) {
    let map = map_loader::load_map("test_map");
    map_info.name = "test_map".to_string();
    map_info.grid_width = map.width;
    map_info.grid_height = map.height;
    map_info.world_width = map.width as f32 * CELL_SIZE;
    map_info.world_height = map.height as f32 * CELL_SIZE;
    map_loader::apply_map(
        map,
        &mut commands, &asset_server,
        &mut objectives_check_inactive_flag,
        &mut emissions_energy_recalculate_all,
        &mut obstacles_grid,
    );
}