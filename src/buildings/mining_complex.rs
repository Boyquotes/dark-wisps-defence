use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, MarkerMiningComplex, TechnicalState};
use crate::common::Z_BUILDING;
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::inventory::resources::DarkOreStock;

pub const MINING_COMPLEX_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };
pub const MINING_COMPLEX_BASE_IMAGE: &str = "buildings/mining_complex.png";

#[derive(Component)]
pub struct MiningComplexDeliveryTimer(pub Timer);

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
        Building::from(BuildingType::MiningComplex),
        TechnicalState::default(),
        MiningComplexDeliveryTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
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
        texture: asset_server.load(MINING_COMPLEX_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(MINING_COMPLEX_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}

pub fn mine_ore_system(
    mut dark_ore_stock: ResMut<DarkOreStock>,
    mut mining_complexes: Query<(&mut MiningComplexDeliveryTimer, &TechnicalState), With<MarkerMiningComplex>>,
    time: Res<Time>,
) {
    for (mut timer, technical_state) in mining_complexes.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            dark_ore_stock.amount += 10;
        }
    }
}