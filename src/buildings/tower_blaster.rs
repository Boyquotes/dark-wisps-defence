use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TowerWispTarget, TowerShootingTimer};
use crate::common_components::Health;
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::grids::obstacles::ObstacleGrid;
use crate::grids::wisps::WispsGrid;
use crate::projectiles::laser_dart::create_laser_dart;
use crate::search::targetfinding::target_find_closest_wisp;
use crate::wisps::components::Wisp;

const TOWER_BLASTER_GRID_WIDTH: i32 = 2;
const TOWER_BLASTER_GRID_HEIGHT: i32 = 2;
const TOWER_BLASTER_WORLD_WIDTH: f32 = CELL_SIZE * TOWER_BLASTER_GRID_WIDTH as f32;
const TOWER_BLASTER_WORLD_HEIGHT: f32 = CELL_SIZE * TOWER_BLASTER_GRID_HEIGHT as f32;

#[derive(Component)]
pub struct MarkerTowerBlaster;

pub fn create_tower_blaster(commands: &mut Commands, grid: &mut ResMut<ObstacleGrid>, grid_position: GridCoords) -> Entity {
    let building = Building {
        grid_imprint: get_tower_blaster_grid_imprint(),
        building_type: BuildingType::Tower(TowerType::Blaster)
    };
    let building_entity = commands.spawn(
        get_tower_blaster_sprite_bundle(grid_position)
    ).insert((
        MarkerTower,
        MarkerTowerBlaster,
        grid_position,
        Health(10000),
        building.clone(),
        TowerShootingTimer::from_seconds(0.2),
        TowerWispTarget::default()
    )).id();
    grid.imprint_building(building, grid_position, building_entity);
    building_entity
}

pub fn get_tower_blaster_sprite_bundle(coords: GridCoords) -> SpriteBundle {
    let world_position = coords.to_world_position().extend(0.);
    SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.8, 0.6, 0.4),
            custom_size: Some(Vec2::new(TOWER_BLASTER_WORLD_WIDTH, TOWER_BLASTER_WORLD_HEIGHT)),
            ..Default::default()
        },
        transform: Transform::from_translation(world_position + Vec3::new(TOWER_BLASTER_WORLD_WIDTH/2., TOWER_BLASTER_WORLD_HEIGHT/2., 0.0)),
        ..Default::default()
    }
}

pub const fn get_tower_blaster_grid_imprint() -> GridImprint {
    GridImprint::Rectangle { width: TOWER_BLASTER_GRID_WIDTH , height: TOWER_BLASTER_GRID_HEIGHT }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_blasters: Query<(&Transform, &mut TowerShootingTimer, &mut TowerWispTarget), With<MarkerTowerBlaster>>,
    mut wisps: Query<&Transform, With<Wisp>>,
) {
    for (transform, mut timer, mut target) in tower_blasters.iter_mut() {
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
    mut tower_blasters: Query<(&GridCoords, &Building, &mut TowerWispTarget), With<MarkerTowerBlaster>>,
    obstacle_grid: Res<ObstacleGrid>,
    wisps_grid: Res<WispsGrid>,
) {
    for (coords, building, mut target) in tower_blasters.iter_mut() {
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