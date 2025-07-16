use lib_grid::grids::emissions::EmissionsGrid;
use lib_grid::grids::obstacles::{Field, ObstacleGrid};
use lib_grid::grids::wisps::WispsGrid;
use lib_grid::search::pathfinding::path_find_energy_beckon;

use crate::effects::wisp_attack::BuilderWispAttackEffect;
use crate::inventory::stats::StatsWispsKilled;
use crate::prelude::*;

use super::components::{Wisp, WispAttackRange, WispChargeAttack, WispState, WispType};
use super::spawning::BuilderWisp;

pub fn move_wisps(
    time: Res<Time>,
    mut wisps_grid: ResMut<WispsGrid>,
    mut wisps: Query<(Entity, &WispState, &Health, &Speed, &mut Transform, &mut GridPath, &mut GridCoords), With<Wisp>>,
) {
    for (entity, wisp_state, health, speed, mut transform, mut grid_path, mut grid_coords) in wisps.iter_mut() {
        if !matches!(*wisp_state, WispState::MovingToTarget) || health.is_dead() { continue; }
        let Some(next_target) = grid_path.next_in_path() else { continue; };
        let curr_world_coords = transform.translation.truncate();
        let interim_target_world_coords = next_target.to_world_position_centered(GridImprint::default());
        let direction = interim_target_world_coords - curr_world_coords;
        let (sx, sy) = (direction.x.signum(), direction.y.signum());
        let wisp_speed = speed.0;
        transform.translation += Vec3::new(sx * time.delta_secs() * wisp_speed, sy * time.delta_secs() * wisp_speed, 0.);
        // If close enough, remove from path.
        if (transform.translation.truncate().distance(interim_target_world_coords)) < 1. {
            grid_path.remove_first();
        }
        // Update grid coords
        let new_coords = GridCoords::from_transform(&transform);
        if new_coords != *grid_coords {
            wisps_grid.wisp_move(*grid_coords, new_coords, entity.into());
            *grid_coords = new_coords;
        }
    }
}

pub fn target_wisps(
    mut wisps_query: Query<(&mut WispState, &mut GridPath, &GridCoords), With<Wisp>>,
    obstacle_grid: Res<ObstacleGrid>,
    emissions_grid: Res<EmissionsGrid>,
) {
    wisps_query.par_iter_mut().for_each(|(mut wisp_state, mut grid_path, grid_coords)| {
        // Retarget is needed when grid has changed or there is no target yet.
        let is_path_outdated = matches!(*wisp_state, WispState::MovingToTarget) && grid_path.grid_version != obstacle_grid.version;
        let need_retarget = is_path_outdated || matches!(*wisp_state, WispState::NeedTarget | WispState::JustSpawned) || matches!(*wisp_state, WispState::Stranded(ref grid_version) if obstacle_grid.version != *grid_version);
        if !need_retarget { return; }

        if let Some(path) = path_find_energy_beckon(&obstacle_grid, &emissions_grid, *grid_coords) {
            *wisp_state = WispState::MovingToTarget;
            grid_path.grid_version = obstacle_grid.version;
            grid_path.path = path.into();
        } else {
            *wisp_state = WispState::Stranded(obstacle_grid.version)
        }
    });
}

pub fn remove_dead_wisps(
    mut commands: Commands,
    mut stock: ResMut<Stock>,
    mut wisps_grid: ResMut<WispsGrid>,
    mut stats_wisps_killed: ResMut<StatsWispsKilled>,
    wisps: Query<(Entity, &Health, &GridCoords, &EssencesContainer), With<Wisp>>,
) {
    for (wisp_entity, health, coords, essences) in wisps.iter() {
        if health.is_dead() {
            wisps_grid.wisp_remove(*coords, wisp_entity.into());
            commands.entity(wisp_entity).despawn();
            // Grant essence
            for container in essences.0.iter() {
                stock.add(container.essence_type.into(), container.amount);
            }
            // Update stats
            stats_wisps_killed.0 += 1;
        }
    }
}

pub fn wisp_charge_attack(
    mut commands: Commands,
    time: Res<Time>,
    obstacle_grid: Res<ObstacleGrid>,
    mut wisps: Query<(&mut WispState, &Health, &Speed, &WispAttackRange, &GridPath, &mut Transform, &mut WispChargeAttack, &GridCoords), (With<Wisp>, Without<Building>)>,
    mut buildings: Query<&mut Health, (With<Building>, Without<Wisp>)>,
) {
    for (mut wisp_state, health, speed, attack_range, grid_path, mut transform, mut attack, grid_coords) in wisps.iter_mut() {
        // --- Validation ---
        if health.is_dead() { continue; }
        // First check if moving wisps should switch to attack mode
        if matches!(*wisp_state, WispState::MovingToTarget) {
            // If wisps is at distance 1 to its target, it's always in range
            if grid_path.distance() == 1 {
                *wisp_state = WispState::Attacking;
            } else if let Some(coords_in_range) = grid_path.at_distance(attack_range.0) {
                // Otherwise, check if the field in the current range is a building
                if obstacle_grid[coords_in_range].is_building() {
                    *wisp_state = WispState::Attacking;
                }
            }
        }
        if !matches!(*wisp_state, WispState::Attacking) { continue; }
        // Then confirm the target still exists
        let Some(target_coords) = grid_path.at_distance(attack_range.0) else { continue; };
        let Field::Building(target_entity, .. ) = obstacle_grid[target_coords] else {
            // If not, then either find new target if we were already at our itended target, or continue moving if we were stopped by an obstacle
            if grid_path.distance() <= attack_range.0 {
                *wisp_state = WispState::NeedTarget;
            } else {
                *wisp_state = WispState::MovingToTarget;
            }
            continue; 
        };
        // --- Charge Attack ---
        // Then execute the attack
        match *attack {
            WispChargeAttack::Charge => {
                // Charge means normal movement, just sped up
                let curr_world_coords = transform.translation.truncate();
                let interim_target_world_coords = target_coords.to_world_position_centered(GridImprint::default());
                let direction = interim_target_world_coords - curr_world_coords;
                let distance = direction.length();
                
                if distance < 1. {
                    // Already close enough, trigger attack
                    *attack = WispChargeAttack::Backoff;
                    commands.queue(BuilderWispAttackEffect::new(transform.translation.xy()));
                    // Deal damage to the building
                    let _ = buildings.get_mut(target_entity).map(|mut health| {
                        health.decrease(1);
                    });
                } else {
                    let wisp_speed = time.delta_secs() * speed.0 * 5.; // Speed up during charge
                    if wisp_speed >= distance {
                        // Would overshoot, just move to target position
                        transform.translation = Vec3::new(interim_target_world_coords.x, interim_target_world_coords.y, transform.translation.z);
                    } else {
                        // Normal movement
                        let normalized_direction = direction / distance;
                        let movement = normalized_direction * wisp_speed;
                        transform.translation += Vec3::new(movement.x, movement.y, 0.);
                    }
                }
            },
            WispChargeAttack::Backoff => {
                // Backoff means to go back half the normal speed to repeat the charge
                let curr_world_coords = transform.translation.truncate();
                let interim_target_world_coords = grid_coords.to_world_position_centered(GridImprint::default());
                let direction = interim_target_world_coords - curr_world_coords;
                let distance = direction.length();
                
                if distance < 1. {
                    // Already close enough, start charging again
                    *attack = WispChargeAttack::Charge;
                } else {
                    let wisp_speed = time.delta_secs() * speed.0 * 0.5;
                    if wisp_speed >= distance {
                        // Would overshoot, just move to target position
                        transform.translation = Vec3::new(interim_target_world_coords.x, interim_target_world_coords.y, transform.translation.z);
                    } else {
                        // Normal movement
                        let normalized_direction = direction / distance;
                        let movement = normalized_direction * wisp_speed;
                        transform.translation += Vec3::new(movement.x, movement.y, 0.);
                    }
                }
            },
        }
    }
}

// For wisps not having any attack defined
pub fn collide_wisps(
    mut commands: Commands,
    wisps: Query<(Entity, &WispState, &GridPath, &Health, &Transform, &GridCoords), (With<Wisp>, Without<Building>)>,
    mut buildings: Query<&mut Health, With<Building>>,
    grid: Res<ObstacleGrid>,
    mut wisps_grid: ResMut<WispsGrid>,
) {
    for (wisp_entity, wisp_state, grid_path, health, transform, coords) in wisps.iter() {
        if !matches!(wisp_state, WispState::MovingToTarget) || health.is_dead() { continue; }
        if !grid_path.is_empty() { continue; }
        let building_entity = match &grid[*coords] {
            Field::Building(entity, ..) => *entity,
            _ => panic!("Expected a building"),
        };
        let mut health = buildings.get_mut(building_entity).unwrap();
        health.decrease(1);
        wisps_grid.wisp_remove(*coords, wisp_entity.into());
        commands.entity(wisp_entity).despawn();
        commands.queue(BuilderWispAttackEffect::new(transform.translation.xy()))
    }
}


pub fn spawn_wisps(
    mut commands: Commands,
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
        let wisp_entity = commands.spawn_empty().id();
        commands.queue(BuilderWisp::new(wisp_entity, WispType::random(), grid_coords));
        wisps_grid.wisp_add(grid_coords, wisp_entity.into());
    }
}