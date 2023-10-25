use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::components::{Building, MarkerTower};
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

const TOWER_BLASTER_GRID_WIDTH: i32 = 2;
const TOWER_BLASTER_GRID_HEIGHT: i32 = 2;
const TOWER_BLASTER_WORLD_WIDTH: f32 = CELL_SIZE * TOWER_BLASTER_GRID_WIDTH as f32;
const TOWER_BLASTER_WORLD_HEIGHT: f32 = CELL_SIZE * TOWER_BLASTER_GRID_HEIGHT as f32;

#[derive(Component)]
pub struct MarkerTowerBlaster;

#[derive(Component)]
pub struct TowerBlasterShootingTimer(Timer);

#[derive(Component, Default)]
pub enum TowerBlasterTarget{
    #[default]
    SearchForNewTarget,
    Wisp(WispEntity),
    NoValidTargets(GridVersion),
}

pub fn create_tower_blaster(commands: &mut Commands, grid: &mut ResMut<ObstacleGrid>, grid_position: GridCoords) -> Entity {
    let imprint = get_tower_blaster_grid_imprint();
    let building_entity = commands.spawn(
        get_tower_blaster_sprite_bundle(grid_position)
    ).insert((
        MarkerTower,
        MarkerTowerBlaster,
        grid_position,
        Health(10000),
        Building {
          grid_imprint: imprint,
          building_type: BuildingType::Tower(TowerType::Blaster)
        },
        TowerBlasterShootingTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        TowerBlasterTarget::default()
    )).id();
    grid.imprint_building(imprint, grid_position, building_entity);
    building_entity
}

pub fn get_tower_blaster_sprite_bundle(coords: GridCoords) -> SpriteBundle {
    let world_position = coords.to_world_coords().extend(0.);
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

pub fn onclick_spawn_system(
    mut commands: Commands,
    mut grid: ResMut<ObstacleGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
) {
    let grid_imprint = match &*grid_object_placer.single() {
        GridObjectPlacer::Building(building) => {
            if !matches!(building.building_type, BuildingType::Tower(TowerType::Blaster)) { return; }
            building.grid_imprint
        }
        _ => { return; }
    };
    let mouse_coords = mouse_info.grid_coords;
    if !mouse_coords.is_in_bounds(grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) && grid.is_imprint_placable(mouse_coords, grid_imprint) {
        // Place the tower blaster
        create_tower_blaster(&mut commands, &mut grid, mouse_coords);
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_blasters: Query<(&Transform, &mut TowerBlasterShootingTimer, &mut TowerBlasterTarget), With<MarkerTowerBlaster>>,
    time: Res<Time>,
    mut wisps: Query<&Transform, With<Wisp>>,
) {
    for (transform, mut timer, mut target) in tower_blasters.iter_mut() {
        let TowerBlasterTarget::Wisp(target_wisp) = *target else { continue; };
        // If timer is paused, try to fire right away, if not - unpause it first.
        // This is necessary for turret to shot right away when spotting first target
        if !timer.0.paused() {
            timer.0.tick(time.delta());
            if !timer.0.just_finished() { continue; }
        } else {
            timer.0.reset();
            timer.0.unpause();
        }
        let Ok(wisp_position) = wisps.get(*target_wisp).map(|target| target.translation.xy()) else {
            // Target wisp does not exist anymore
            *target = TowerBlasterTarget::SearchForNewTarget;
            timer.0.pause();
            continue;
        };

        create_laser_dart(&mut commands, transform.translation, target_wisp, (wisp_position - transform.translation.xy()).normalize());
    }
}

pub fn targeting_system(
    mut tower_blasters: Query<(&GridCoords, &Building, &mut TowerBlasterTarget), With<MarkerTowerBlaster>>,
    obstacle_grid: Res<ObstacleGrid>,
    wisps_grid: Res<WispsGrid>,
) {
    for (coords, building, mut target) in tower_blasters.iter_mut() {
        match *target {
            TowerBlasterTarget::Wisp(_) => continue,
            TowerBlasterTarget::NoValidTargets(grid_version) => {
                if grid_version == wisps_grid.version {
                    continue;
                }
            },
            TowerBlasterTarget::SearchForNewTarget => {},
        }
        if let Some((_a, target_wisp)) = target_find_closest_wisp(
            &obstacle_grid,
            &wisps_grid,
            building.grid_imprint.covered_coords(*coords),
        15,
        false
        ) {
            *target = TowerBlasterTarget::Wisp(target_wisp);
        } else {
            *target = TowerBlasterTarget::NoValidTargets(wisps_grid.version);
        }
    }
}