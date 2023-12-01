use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TechnicalState, TowerRange, TowerShootingTimer, TowerWispTarget};
use crate::common::Z_BUILDING;
use crate::common_components::{Health};
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::projectiles::cannonball::create_cannonball;
use crate::wisps::components::{Target, Wisp};
use crate::wisps::spawning::WISP_GRID_IMPRINT;
pub const TOWER_CANNON_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };
pub const TOWER_CANNON_BASE_IMAGE: &str = "buildings/tower_cannon.png";

#[derive(Component)]
pub struct MarkerTowerCannon;

pub fn create_tower_cannon(
    commands: &mut Commands,
    asset_server: &AssetServer,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    energy_supply_grid: &EnergySupplyGrid,
    grid_position: GridCoords,
) -> Entity {
    let building_entity = commands.spawn(
        get_tower_cannon_sprite_bundle(grid_position, asset_server),
    ).insert((
        MarkerTower,
        MarkerTowerCannon,
        grid_position,
        Health(10000),
        TowerRange(15),
        Building::from(BuildingType::Tower(TowerType::Cannon)),
        TowerShootingTimer::from_seconds(2.0),
        TowerWispTarget::default(),
        TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, TOWER_CANNON_GRID_IMPRINT) },
    )).id();
    obstacle_grid.imprint(grid_position, Field::Building(building_entity, BuildingType::Tower(TowerType::Cannon)), TOWER_CANNON_GRID_IMPRINT);
    building_entity
}

pub fn get_tower_cannon_sprite_bundle(coords: GridCoords, asset_server: &AssetServer,) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(TOWER_CANNON_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_CANNON_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(TOWER_CANNON_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_cannons: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget), With<MarkerTowerCannon>>,
    wisps: Query<(&Target, &GridCoords), With<Wisp>>,
) {
    for (transform, technical_state, mut timer, mut target) in tower_cannons.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok((wisp_target, wisp_coords)) = wisps.get(*target_wisp) else {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        let target_world_position = wisp_target.grid_path
            .as_ref()
            .map_or(
                wisp_coords.to_world_position_centered(WISP_GRID_IMPRINT),
                |path| {
                    path
                        .first()
                        .map_or(
                            wisp_coords.to_world_position_centered(WISP_GRID_IMPRINT),
                            |coords| coords.to_world_position_centered(WISP_GRID_IMPRINT)
                        )
                }
            );

        create_cannonball(&mut commands, transform.translation.xy(), target_world_position);
        timer.0.reset();
    }
}
