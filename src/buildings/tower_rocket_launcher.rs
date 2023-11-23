use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TechnicalState, TowerRange, TowerShootingTimer, TowerWispTarget};
use crate::common_components::{Health};
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::projectiles::cannonball::create_cannonball;
use crate::projectiles::rocket::create_rocket;
use crate::wisps::components::{Target, Wisp};

pub const TOWER_ROCKET_LAUNCHER_GRID_WIDTH: i32 = 3;
pub const TOWER_ROCKET_LAUNCHER_GRID_HEIGHT: i32 = 3;
pub const TOWER_ROCKET_LAUNCHER_WORLD_WIDTH: f32 = CELL_SIZE * TOWER_ROCKET_LAUNCHER_GRID_WIDTH as f32;
pub const TOWER_ROCKET_LAUNCHER_WORLD_HEIGHT: f32 = CELL_SIZE * TOWER_ROCKET_LAUNCHER_GRID_HEIGHT as f32;
pub const TOWER_ROCKET_LAUNCHER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: TOWER_ROCKET_LAUNCHER_GRID_WIDTH , height: TOWER_ROCKET_LAUNCHER_GRID_HEIGHT };

#[derive(Component)]
pub struct MarkerTowerRocketLauncher;

pub fn create_tower_rocket_launcher(
    commands: &mut Commands,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    energy_supply_grid: &EnergySupplyGrid,
    grid_position: GridCoords,
) -> Entity {
    let building = Building {
        grid_imprint: TOWER_ROCKET_LAUNCHER_GRID_IMPRINT,
        building_type: BuildingType::Tower(TowerType::RocketLauncher)
    };
    let building_entity = commands.spawn(
        get_tower_rocket_launcher_sprite_bundle(grid_position)
    ).insert((
        MarkerTower,
        MarkerTowerRocketLauncher,
        grid_position,
        Health(10000),
        TowerRange(30),
        building.clone(),
        TowerShootingTimer::from_seconds(2.0),
        TowerWispTarget::default(),
        TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, building.grid_imprint) },
    )).id();
    obstacle_grid.imprint(grid_position, Field::Building(building_entity, building.building_type), TOWER_ROCKET_LAUNCHER_GRID_IMPRINT);
    building_entity
}

pub fn get_tower_rocket_launcher_sprite_bundle(coords: GridCoords) -> SpriteBundle {
    let world_position = coords.to_world_position().extend(0.);
    SpriteBundle {
        sprite: Sprite {
            color: Color::rgb_u8(80, 45, 104),
            custom_size: Some(Vec2::new(TOWER_ROCKET_LAUNCHER_WORLD_WIDTH, TOWER_ROCKET_LAUNCHER_WORLD_HEIGHT)),
            ..Default::default()
        },
        transform: Transform::from_translation(world_position + Vec3::new(TOWER_ROCKET_LAUNCHER_WORLD_WIDTH/2., TOWER_ROCKET_LAUNCHER_WORLD_HEIGHT/2., 0.0)),
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

        create_rocket(&mut commands, transform.translation, target_wisp);
        timer.0.reset();
    }
}
