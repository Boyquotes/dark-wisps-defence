use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::components::{Building, MarkerTower};
use crate::common_components::{Health, TargetVector};
use crate::grids::common::CELL_SIZE;
use crate::grids::obstacles::{ObstacleGrid, GridCoords, GridImprint};
use crate::mouse::MouseInfo;
use crate::projectiles::laser_dart::create_laser_dart;
use crate::ui::grid_object_placer::GridObjectPlacer;

const TOWER_BLASTER_GRID_WIDTH: i32 = 2;
const TOWER_BLASTER_GRID_HEIGHT: i32 = 2;
const TOWER_BLASTER_WORLD_WIDTH: f32 = CELL_SIZE * TOWER_BLASTER_GRID_WIDTH as f32;
const TOWER_BLASTER_WORLD_HEIGHT: f32 = CELL_SIZE * TOWER_BLASTER_GRID_HEIGHT as f32;

#[derive(Component)]
pub struct MarkerTowerBlaster;

#[derive(Component)]
pub struct TowerBlasterShootingTimer(Timer);

pub fn create_tower_blaster(commands: &mut Commands, grid: &mut ResMut<ObstacleGrid>, grid_position: GridCoords) -> Entity {
    let imprint = get_tower_blaster_grid_imprint();
    let building_entity = commands.spawn(
        get_tower_blaster_sprite_bundle(grid_position)
    ).insert(
        (MarkerTower, MarkerTowerBlaster)
    ).insert(
        grid_position
    ).insert(
        Health(10000)
    ).insert(
      Building {
          grid_imprint: imprint,
          building_type: BuildingType::Tower(TowerType::Blaster)
      }
    ).insert(
        TowerBlasterShootingTimer(Timer::from_seconds(0.5, TimerMode::Repeating))
    ).id();
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

pub fn get_tower_blaster_grid_imprint() -> GridImprint {
    GridImprint::Rectangle { width: TOWER_BLASTER_GRID_WIDTH , height: TOWER_BLASTER_GRID_HEIGHT }
}

pub fn onclick_tower_blaster_spawn_system(
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

pub fn tower_blaster_shooting_system(
    mut commands: Commands,
    mut tower_blasters: Query<(&Transform, &mut TowerBlasterShootingTimer), With<MarkerTowerBlaster>>,
    time: Res<Time>,
    mouse_info: Res<MouseInfo>,
) {
    for (transform, mut timer) in tower_blasters.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            create_laser_dart(&mut commands, transform.translation, TargetVector((mouse_info.world_position - transform.translation.xy()).normalize()));
        }
    }
}