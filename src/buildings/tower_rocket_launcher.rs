use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TechnicalState, TowerRange, TowerShootingTimer, TowerWispTarget};
use crate::common::Z_BUILDING;
use crate::common_components::{Health};
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::projectiles::rocket::create_rocket;
use crate::wisps::components::{Target, Wisp};

pub const TOWER_ROCKET_LAUNCHER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };

pub const TOWER_ROCKET_LAUNCHER_BASE_IMAGE: &str = "buildings/tower_rocket_launcher.png";

#[derive(Component)]
pub struct MarkerTowerRocketLauncher;

pub fn create_tower_rocket_launcher(
    commands: &mut Commands,
    asset_server: &AssetServer,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    energy_supply_grid: &EnergySupplyGrid,
    grid_position: GridCoords,
) -> Entity {
    let building_entity = commands.spawn(
        get_tower_rocket_launcher_sprite_bundle(asset_server, grid_position)
    ).insert((
        MarkerTower,
        MarkerTowerRocketLauncher,
        grid_position,
        Health(10000),
        TowerRange(30),
        Building::from(BuildingType::Tower(TowerType::RocketLauncher)),
        TowerShootingTimer::from_seconds(2.0),
        TowerWispTarget::default(),
        TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, TOWER_ROCKET_LAUNCHER_GRID_IMPRINT) },
    )).id();
    obstacle_grid.imprint(grid_position, Field::Building(building_entity, BuildingType::Tower(TowerType::RocketLauncher)), TOWER_ROCKET_LAUNCHER_GRID_IMPRINT);
    building_entity
}

pub fn get_tower_rocket_launcher_sprite_bundle(asset_server: &AssetServer, coords: GridCoords) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(TOWER_ROCKET_LAUNCHER_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_ROCKET_LAUNCHER_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(TOWER_ROCKET_LAUNCHER_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_rocket_launchers: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget), With<MarkerTowerRocketLauncher>>,
    wisps: Query<(&Target, &GridCoords), With<Wisp>>,
) {
    for (transform, technical_state, mut timer, mut target) in tower_rocket_launchers.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        if wisps.get(*target_wisp).is_err() {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        create_rocket(&mut commands, transform.translation.xy(), target_wisp);
        timer.0.reset();
    }
}
