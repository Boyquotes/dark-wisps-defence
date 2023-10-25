use bevy::prelude::*;
use nanorand::Rng;
use crate::buildings::common_components::Building;
use crate::common::TargetType;
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridType};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::grids::wisps::WispsGrid;
use crate::is_game_mode;
use crate::search::pathfinding::path_find_closest_building;
use crate::wisps::components::{Target, Wisp};
use crate::wisps::spawning::spawn_wisp;

pub struct WispsPlugin;
impl Plugin for WispsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            move_wisps,
            target_wisps,
            collide_wisps,
            remove_dead_wisps,
        ));
        app.add_systems(Update, spawn_wisps.run_if(is_game_mode));
    }
}

pub fn move_wisps(
    mut wisps: Query<(Entity, &Health, &mut Transform, &mut Target, &mut GridCoords), With<Wisp>>,
    time: Res<Time>,
    mut wisps_grid: ResMut<WispsGrid>,
) {
    for (entity, health, mut transform, mut target, mut grid_coords) in wisps.iter_mut() {
        if !target.is_on_its_path() || health.is_dead() { continue; }
        if let Some(grid_path) = &mut target.grid_path {
            let next_target = grid_path.first().unwrap();
            let curr_world_coords = transform.translation.truncate();
            let interim_target_world_coords = next_target.to_world_coords_centered();
            let direction = interim_target_world_coords - curr_world_coords;
            let (sx, sy) = (direction.x.signum(), direction.y.signum());
            transform.translation += Vec3::new(sx * time.delta_seconds() * 30., sy * time.delta_seconds() * 30., 0.);
            // If close enough, remove from path.
            if (transform.translation.truncate().distance(interim_target_world_coords)) < 1. {
                grid_path.remove(0);
                if grid_path.is_empty() {
                    target.grid_path = None;
                }
            }
            let new_coords = GridCoords::from_transform(&transform);
            if new_coords != *grid_coords {
                wisps_grid.wisp_move(*grid_coords, new_coords, entity.into());
                *grid_coords = new_coords;
            }
        }
    }
}

pub fn target_wisps(mut query: Query<(&mut Target, &GridCoords), With<Wisp>>, grid: Res<ObstacleGrid>) {
    for (mut target, grid_coords) in query.iter_mut() {
        // First check if there was anything that would invalidate existing targeting.
        match target.target_type {
            TargetType::None | TargetType::DynamicObject(_) => {},
            TargetType::Field{grid_version, ..} => {
                if grid_version != grid.version {
                    target.target_type = TargetType::None;
                }
            }
            TargetType::Unreachable{grid_type, grid_version} => {
                match grid_type {
                    GridType::Obstacles => {
                        if grid_version != grid.version {
                            target.target_type = TargetType::None;
                        }
                    }
                }
            }
        }
        // Then check if the wisp is eligible for new targeting.
        if target.is_on_its_path() || target.is_at_destination() || target.is_unreachable() { continue; }

        if let Some(path) = path_find_closest_building(&grid, *grid_coords) {
            target.target_type = TargetType::Field{coords: *path.last().unwrap(), grid_version: grid.version};
            target.grid_path = Some(path);
        } else {
            target.target_type = TargetType::Unreachable {grid_type: GridType::Obstacles, grid_version: grid.version};
            target.grid_path = Some(Vec::new());
        }
    }
}

pub fn remove_dead_wisps(
    mut commands: Commands,
    wisps: Query<(Entity, &Health, &GridCoords), With<Wisp>>,
    mut wisps_grid: ResMut<WispsGrid>,
) {
    for (wisp_entity, health, coords) in wisps.iter() {
        if health.is_dead() {
            wisps_grid.wisp_remove(*coords, wisp_entity.into());
            commands.entity(wisp_entity).despawn();
        }
    }
}

pub fn collide_wisps(
    mut commands: Commands,
    wisps: Query<(Entity, &Target, &Health), (With<Wisp>, Without<Building>)>,
    mut buildings: Query<&mut Health, With<Building>>,
    grid: Res<ObstacleGrid>,
    mut wisps_grid: ResMut<WispsGrid>,
) {
    for (wisp_entity, target, health) in wisps.iter() {
        if !target.is_at_destination() || health.is_dead() { continue; }
        match &target.target_type {
            TargetType::Field{coords, ..} => {
                let building_entity = match &grid[*coords] {
                    Field::Building(entity) => *entity,
                    _ => panic!("Expected a building"),
                };
                let mut health = buildings.get_mut(building_entity).unwrap();
                health.decrease(1);
                wisps_grid.wisp_remove(*coords, wisp_entity.into());
                commands.entity(wisp_entity).despawn();
            }
            _ => panic!("Expected a field"),
        }
    }
}


pub fn spawn_wisps(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    obstacle_grid: Res<ObstacleGrid>,
    mut wisps_grid: ResMut<WispsGrid>,
) {
    let mut rng = nanorand::tls_rng();
    if rng.generate::<f32>() < 0.01 {
        let grid_coords = {
            let chance = rng.generate::<f32>();
            if chance < 0.25 {
                GridCoords{x: rng.generate_range(1..=obstacle_grid.width), y: 0} // Nano-rand is off by 1! this is (0..grid.width)
            } else if chance < 0.5 {
                GridCoords{x: 0, y: rng.generate_range(1..=obstacle_grid.height)}
            } else if chance < 0.75 {
                GridCoords{x: rng.generate_range(1..=obstacle_grid.width), y: obstacle_grid.height - 1}
            } else {
                GridCoords{x: obstacle_grid.width - 1, y: rng.generate_range(1..=obstacle_grid.height)}
            }
        };
        let new_wisp = spawn_wisp(&mut commands, &mut meshes, &mut materials, grid_coords);
        wisps_grid.wisp_add(grid_coords, new_wisp.into());
    }
}