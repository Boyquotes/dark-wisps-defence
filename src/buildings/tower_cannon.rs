use std::time::Duration;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TowerShootingTimer, TowerWispTarget};
use crate::common_components::{Health};
use crate::grids::base::GridVersion;
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::grids::obstacles::ObstacleGrid;
use crate::grids::wisps::WispsGrid;
use crate::mouse::MouseInfo;
use crate::projectiles::laser_dart::create_laser_dart;
use crate::search::targetfinding::target_find_closest_wisp;
use crate::ui::grid_object_placer::GridObjectPlacer;
use crate::wisps::components::{Wisp, WispEntity};

const TOWER_CANNON_GRID_WIDTH: i32 = 3;
const TOWER_CANNON_GRID_HEIGHT: i32 = 3;
const TOWER_CANNON_WORLD_WIDTH: f32 = CELL_SIZE * TOWER_CANNON_GRID_WIDTH as f32;
const TOWER_CANNON_WORLD_HEIGHT: f32 = CELL_SIZE * TOWER_CANNON_GRID_HEIGHT as f32;

#[derive(Component)]
pub struct MarkerTowerCannon;

pub fn create_tower_cannon(commands: &mut Commands, grid: &mut ResMut<ObstacleGrid>, grid_position: GridCoords) -> Entity {
    let imprint = get_tower_cannon_grid_imprint();
    let building_entity = commands.spawn(
        get_tower_cannon_sprite_bundle(grid_position)
    ).insert((
        MarkerTower,
        MarkerTowerCannon,
        grid_position,
        Health(10000),
        Building {
          grid_imprint: imprint,
          building_type: BuildingType::Tower(TowerType::Cannon)
        },
        TowerShootingTimer::from_seconds(2.0),
        TowerWispTarget::default()
    )).id();
    grid.imprint_building(imprint, grid_position, building_entity);
    building_entity
}

pub fn get_tower_cannon_sprite_bundle(coords: GridCoords) -> SpriteBundle {
    let world_position = coords.to_world_coords().extend(0.);
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
    mut tower_cannons: Query<(&Transform, &mut TowerShootingTimer, &mut TowerWispTarget), With<MarkerTowerCannon>>,
    mut wisps: Query<&Transform, With<Wisp>>,
) {
    for (transform, mut timer, mut target) in tower_cannons.iter_mut() {
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok(wisp_position) = wisps.get(*target_wisp).map(|target| target.translation.xy()) else {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        create_laser_dart(&mut commands, transform.translation, target_wisp, (wisp_position - transform.translation.xy()).normalize());
        timer.0.reset();
    }
}

pub fn targeting_system(
    mut tower_cannons: Query<(&GridCoords, &Building, &mut TowerWispTarget), With<MarkerTowerCannon>>,
    obstacle_grid: Res<ObstacleGrid>,
    wisps_grid: Res<WispsGrid>,
) {
    for (coords, building, mut target) in tower_cannons.iter_mut() {
        match *target {
            TowerWispTarget::Wisp(_) => continue,
            TowerWispTarget::NoValidTargets(grid_version) => {
                if grid_version == wisps_grid.version {
                    continue;
                }
            },
            TowerWispTarget::SearchForNewTarget => {},
        }
        if let Some((_a, target_wisp)) = target_find_closest_wisp(
            &obstacle_grid,
            &wisps_grid,
            building.grid_imprint.covered_coords(*coords),
        15,
        true,
        ) {
            *target = TowerWispTarget::Wisp(target_wisp);
        } else {
            *target = TowerWispTarget::NoValidTargets(wisps_grid.version);
        }
    }
}