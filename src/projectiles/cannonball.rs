use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::common_components::{Health};
use crate::grids::common::GridCoords;
use crate::grids::wisps::WispsGrid;
use crate::projectiles::components::MarkerProjectile;
use crate::search::common::ALL_DIRECTIONS;
use crate::wisps::components::{Wisp};

#[derive(Component)]
pub struct MarkerCannonball;

// Cannonball follows Wisp, and if the wisp no longer exists, follows the target vector
#[derive(Component, Default)]
pub struct CannonballTarget(Vec2);

pub fn create_cannonball(commands: &mut Commands, world_position: Vec3, target_position: Vec2) -> Entity {
    let entity = commands.spawn(
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..Default::default()
            },
            transform: Transform {
                translation: world_position,
                ..Default::default()
            },
            ..Default::default()
        }
    ).insert(
        (MarkerProjectile, MarkerCannonball)
    ).insert(
        CannonballTarget(target_position)
    ).id();
    entity
}

pub fn cannonball_move_system(
    mut cannonballs: Query<(&mut Transform, &CannonballTarget), With<MarkerCannonball>>,
    time: Res<Time>,
) {
    for (mut transform, target) in cannonballs.iter_mut() {
        let direction_vector = (target.0.extend(0.) - transform.translation).normalize();
        transform.translation += direction_vector * time.delta_seconds() * 300.;
    }
}

pub fn cannonball_hit_system(
    mut commands: Commands,
    cannonballs: Query<(Entity, &Transform, &CannonballTarget), With<MarkerCannonball>>,
    wisps_grid: Res<WispsGrid>,
    mut wisps: Query<&mut Health, With<Wisp>>,
) {
    for (entity, cannonball_transform, target) in cannonballs.iter() {
        if cannonball_transform.translation.xy().distance(target.0) > 1. { continue; }

        let coords = GridCoords::from_transform(&cannonball_transform);
        for (dx, dy) in ALL_DIRECTIONS.iter().chain(&[(0, 0)]) {
            let blast_zone_coords = coords.shifted((*dx, *dy));
            if !blast_zone_coords.is_in_bounds(wisps_grid.bounds()) { continue; }
            let wisps_in_coords = &wisps_grid[blast_zone_coords];
            for wisp in wisps_in_coords {
                let Ok(mut health) = wisps.get_mut(**wisp) else { continue }; // May not find wisp if the wisp spawned at the same frame.
                health.decrease(100);
            }
        }
        commands.entity(entity).despawn();
    }
}