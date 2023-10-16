mod camera;
mod grid;
mod common;
mod buildings;
mod wisps;
mod common_components;
mod pathfinding;
mod map_loader;
mod map_objects;
mod ui;
mod map_editor;
mod mouse;
mod utils;

use bevy::prelude::*;
use crate::grid::{ObstacleGrid};
use crate::map_editor::MapInfo;

fn main() {
    println!("Hello, world!");
    let mut grid = ObstacleGrid::new_empty();
    grid.resize_and_reset(10, 10);
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            camera::CameraPlugin,
            grid::GridPlugin,
            wisps::systems::WispsPlugin,
            ui::UiPlugin,
            map_editor::MapEditorPlugin,
            mouse::MousePlugin,
            map_objects::MapObjectsPlugin,
            buildings::BuildingsPlugin,
        ))
        .insert_resource(GameConfig{
            mode: GameMode::Editor,
        })
        .add_systems(Startup, generate_default_map)
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
    mut grid: ResMut<ObstacleGrid>,
    mut map_info: ResMut<MapInfo>,
) {
    let map = map_loader::load_map("test_map");
    map_info.name = "test_map".to_string();
    map_info.width = map.width;
    map_info.height = map.height;
    map_loader::apply_map(map, &mut commands, &mut grid);
}