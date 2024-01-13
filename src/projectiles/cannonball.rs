use std::f32::consts::PI;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::common::Z_PROJECTILE;
use crate::common_components::{Health};
use crate::effects::explosions::create_explosion;
use crate::grids::common::GridCoords;
use crate::grids::wisps::WispsGrid;
use crate::projectiles::components::MarkerProjectile;
use crate::search::common::ALL_DIRECTIONS;
use crate::wisps::components::{Wisp};

pub const CANNONBALL_BASE_IMAGE: &str = "projectiles/cannonball.png";

#[derive(Component)]
pub struct MarkerCannonball;

// Cannonball follows Wisp, and if the wisp no longer exists, follows to the target position
#[derive(Component, Default)]
pub struct CannonballTarget{
    pub initial_distance: f32,
    pub target_position: Vec2,
}

#[derive(Bundle)]
pub struct BundleCannonball {
    pub sprite: SpriteBundle,
    pub marker_projectile: MarkerProjectile,
    pub marker_cannonball: MarkerCannonball,
    pub cannonball_target: CannonballTarget,
}

impl BundleCannonball {
    pub fn new(world_position: Vec2, target_position: Vec2, asset_server: &AssetServer) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(8.0, 8.0)),
                    ..Default::default()
                },
                texture: asset_server.load(CANNONBALL_BASE_IMAGE),
                transform: Transform {
                    translation: world_position.extend(Z_PROJECTILE),
                    ..Default::default()
                },
                ..Default::default()
            },
            marker_projectile: MarkerProjectile,
            marker_cannonball: MarkerCannonball,
            cannonball_target: CannonballTarget {
                initial_distance: world_position.distance(target_position),
                target_position,
            },
        }
    }
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands.spawn(self).id()
    }
}

pub fn cannonball_move_system(
    mut cannonballs: Query<(&mut Transform, &CannonballTarget), With<MarkerCannonball>>,
    time: Res<Time>,
) {
    for (mut transform, target) in cannonballs.iter_mut() {
        let direction_vector = (target.target_position - transform.translation.xy()).normalize();
        let move_distance = direction_vector * time.delta_seconds() * 200.;

        let remaining_distance = (transform.translation.xy() + move_distance).distance(target.target_position);

        // Calculate the progress as a value between 0 and 1
        let progress = 1. - remaining_distance / target.initial_distance;

        // Determine the scaling factor based on progress, applying a sine function for non-linearity
        let scale_factor = if progress <= 0.5 {
            1.0 + (PI * progress).sin()  // Non-linear scale up in the first half
        } else {
            (PI * (1.0 - progress)).sin() + 1.0  // Non-linear scale down in the second half
        };
        transform.scale = Vec3::splat(scale_factor);

        // Move the cannonball
        transform.translation += move_distance.extend(0.);
    }
}

pub fn cannonball_hit_system(
    mut commands: Commands,
    cannonballs: Query<(Entity, &Transform, &CannonballTarget), With<MarkerCannonball>>,
    wisps_grid: Res<WispsGrid>,
    mut wisps: Query<&mut Health, With<Wisp>>,
) {
    for (entity, cannonball_transform, target) in cannonballs.iter() {
        if cannonball_transform.translation.xy().distance(target.target_position) > 1. { continue; }

        let coords = GridCoords::from_transform(&cannonball_transform);
        for (dx, dy) in ALL_DIRECTIONS.iter().chain(&[(0, 0)]) {
            let blast_zone_coords = coords.shifted((*dx, *dy));
            if !blast_zone_coords.is_in_bounds(wisps_grid.bounds()) { continue; }

            create_explosion(&mut commands, blast_zone_coords);

            let wisps_in_coords = &wisps_grid[blast_zone_coords];
            for wisp in wisps_in_coords {
                let Ok(mut health) = wisps.get_mut(**wisp) else { continue }; // May not find wisp if the wisp spawned at the same frame.
                health.decrease(100);
            }
        }
        commands.entity(entity).despawn();
    }
}