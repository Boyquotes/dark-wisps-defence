use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TechnicalState, TowerRange, TowerShootingTimer, TowerWispTarget};
use crate::common_components::{Health};
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::ObstacleGrid;
use crate::projectiles::cannonball::create_cannonball;
use crate::wisps::components::{Target, Wisp};

const TOWER_CANNON_GRID_WIDTH: i32 = 3;
const TOWER_CANNON_GRID_HEIGHT: i32 = 3;
const TOWER_CANNON_WORLD_WIDTH: f32 = CELL_SIZE * TOWER_CANNON_GRID_WIDTH as f32;
const TOWER_CANNON_WORLD_HEIGHT: f32 = CELL_SIZE * TOWER_CANNON_GRID_HEIGHT as f32;

#[derive(Component)]
pub struct MarkerTowerCannon;

pub fn create_tower_cannon(
    commands: &mut Commands,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    energy_supply_grid: &EnergySupplyGrid,
    grid_position: GridCoords,
) -> Entity {
    let building = Building {
        grid_imprint: get_tower_cannon_grid_imprint(),
        building_type: BuildingType::Tower(TowerType::Cannon)
    };
    let building_entity = commands.spawn(
        get_tower_cannon_sprite_bundle(grid_position)
    ).insert((
        MarkerTower,
        MarkerTowerCannon,
        grid_position,
        Health(10000),
        TowerRange(15),
        building.clone(),
        TowerShootingTimer::from_seconds(2.0),
        TowerWispTarget::default(),
        TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, building.grid_imprint) },
    )).id();
    obstacle_grid.imprint_building(building, grid_position, building_entity);
    building_entity
}

pub fn get_tower_cannon_sprite_bundle(coords: GridCoords) -> SpriteBundle {
    let world_position = coords.to_world_position().extend(0.);
    SpriteBundle {
        sprite: Sprite {
            color: Color::rgb_u8(30, 135, 104),
            custom_size: Some(Vec2::new(TOWER_CANNON_WORLD_WIDTH, TOWER_CANNON_WORLD_HEIGHT)),
            ..Default::default()
        },
        transform: Transform::from_translation(world_position + Vec3::new(TOWER_CANNON_WORLD_WIDTH/2., TOWER_CANNON_WORLD_HEIGHT/2., 0.0)),
        ..Default::default()
    }
}

pub const fn get_tower_cannon_grid_imprint() -> GridImprint {
    GridImprint::Rectangle { width: TOWER_CANNON_GRID_WIDTH , height: TOWER_CANNON_GRID_HEIGHT }
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

        let target_world_position = wisp_target.grid_path.as_ref().map_or(wisp_coords.to_world_position_centered(), |path| {
            path.first().map_or(wisp_coords.to_world_position_centered(), |coords| coords.to_world_position_centered())
        });

        create_cannonball(&mut commands, transform.translation, target_world_position);
        timer.0.reset();
    }
}
