use bevy::prelude::*;
use nanorand::Rng;
use crate::buildings::components::Building;
use crate::common::TargetType;
use crate::common_components::Health;
use crate::grid::{Field, ObstacleGrid, GridCoords, GridType};
use crate::is_game_mode;
use crate::pathfinding::path_find_closest_building;
use crate::wisps::components::{Target, Wisp};
use crate::wisps::spawning::spawn_wisp;

pub struct WispsPlugin;
impl Plugin for WispsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_wisps);
        app.add_systems(Update, target_wisps);
        app.add_systems(Update, collide_wisps);
        app.add_systems(Update, spawn_wisps.run_if(is_game_mode));
    }
}

pub fn move_wisps(mut query: Query<(&mut Transform, &mut Target, &mut GridCoords), With<Wisp>>, time: Res<Time>) {
    for (mut transform, mut target, mut grid_coords) in query.iter_mut() {
        if !target.is_on_its_path() { continue; }
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
            *grid_coords = GridCoords::from_transform(&transform);
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
                    GridType::Obstacle => {
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
            target.target_type = TargetType::Unreachable {grid_type: GridType::Obstacle, grid_version: grid.version};
            target.grid_path = Some(Vec::new());
        }
    }
}

pub fn collide_wisps(
    mut commands: Commands,
    wisps: Query<(&Target, Entity), With<Wisp>>,
    mut buildings: Query<&mut Health, With<Building>>,
    grid: Res<ObstacleGrid>,
) {
    for (target, wisp_entity) in wisps.iter() {
        if !target.is_at_destination() { continue; }
        match &target.target_type {
            TargetType::Field{coords, ..} => {
                let building_entity = match &grid[*coords] {
                    Field::Building(entity) => *entity,
                    _ => panic!("Expected a building"),
                };
                let mut health = buildings.get_mut(building_entity).unwrap();
                health.0 -= 1;
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
    grid: Res<ObstacleGrid>,
) {
    let mut rng = nanorand::tls_rng();
    if rng.generate::<f32>() < 0.01 {
        let grid_coords = {
            let chance = rng.generate::<f32>();
            if chance < 0.25 {
                GridCoords{x: rng.generate_range(1..=grid.width), y: 0} // Nano-rand is off by 1! this is (0..grid.width)
            } else if chance < 0.5 {
                GridCoords{x: 0, y: rng.generate_range(1..=grid.height)}
            } else if chance < 0.75 {
                GridCoords{x: rng.generate_range(1..=grid.width), y: grid.height - 1}
            } else {
                GridCoords{x: grid.width - 1, y: rng.generate_range(1..=grid.height)}
            }
        };
        spawn_wisp(&mut commands, &mut meshes, &mut materials, grid_coords);
    }
}