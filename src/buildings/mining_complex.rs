use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, MarkerMiningComplex, TechnicalState};
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::obstacles::{Field, ObstacleGrid};

pub const MINING_COMPLEX_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };

pub fn create_mining_complex(
    commands: &mut Commands,
    asset_server: &AssetServer,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
    dark_ore: Entity,
) -> Entity {
    let mining_complex = commands.spawn((
        get_mining_complex_sprite_bundle(grid_position, asset_server),
        MarkerMiningComplex,
        grid_position,
        Health(10000),
        Building {
            grid_imprint: MINING_COMPLEX_GRID_IMPRINT,
            building_type: BuildingType::MiningComplex
        },
        TechnicalState::default(),
    )).id();
    obstacle_grid.imprint(grid_position, Field::MiningComplex {dark_ore, mining_complex}, MINING_COMPLEX_GRID_IMPRINT);
    mining_complex
}

pub fn get_mining_complex_sprite_bundle(coords: GridCoords, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(MINING_COMPLEX_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load("buildings/mining_complex.png"),
        transform: Transform::from_translation(coords.to_world_position_centered(MINING_COMPLEX_GRID_IMPRINT).extend(0.)),
        ..Default::default()
    }
}