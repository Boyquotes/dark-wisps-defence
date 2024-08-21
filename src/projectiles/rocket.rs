use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::common::{Z_PROJECTILE, Z_PROJECTILE_UNDER};
use crate::common_components::{Health};
use crate::grids::common::GridCoords;
use crate::grids::wisps::WispsGrid;
use crate::projectiles::components::MarkerProjectile;
use crate::search::common::ALL_DIRECTIONS;
use crate::wisps::components::{Wisp, WispEntity};
use crate::effects::explosions::BuilderExplosion;

pub const ROCKET_BASE_IMAGE: &str = "projectiles/rocket.png";
pub const ROCKET_EXHAUST_IMAGE: &str = "projectiles/rocket_exhaust.png";

#[derive(Component)]
pub struct MarkerRocket {
    pub exhaust: Entity,
}
#[derive(Component)]
pub struct MarkerRocketExhaust;

// Rocket follows Wisp, and if the wisp no longer exists, looks for another target
#[derive(Component)]
pub struct RocketTarget(pub WispEntity);

#[derive(Bundle)]
struct BundleRocketExhaust {
    pub sprite: SpriteBundle,
    pub marker_rocket_exhaust: MarkerRocketExhaust,
}
#[derive(Bundle)]
struct BundleRocketBase {
    pub sprite: SpriteBundle,
    pub marker_projectile: MarkerProjectile,
    pub marker_rocket: MarkerRocket,
    pub rocket_target: RocketTarget,
}
pub struct BuilderRocket {
    exhaust: BundleRocketExhaust,
    base: BundleRocketBase,
}

impl BuilderRocket {
    pub fn new(world_position: Vec2, rotation: Quat, target_wisp: WispEntity, asset_server: &AssetServer) -> Self {
        let exhaust = BundleRocketExhaust {
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(10.0, 6.25)),
                    anchor: Anchor::Custom(Vec2::new(0.75, 0.)),
                    ..Default::default()
                },
                texture: asset_server.load(ROCKET_EXHAUST_IMAGE),
                transform: Transform {
                    translation: Vec2::ZERO.extend(Z_PROJECTILE_UNDER),
                    ..Default::default()
                },
                ..Default::default()
            },
            marker_rocket_exhaust: MarkerRocketExhaust,
        };
        let base = BundleRocketBase {
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(20.0, 10.0)),
                    ..Default::default()
                },
                texture: asset_server.load(ROCKET_BASE_IMAGE),
                transform: Transform {
                    translation: world_position.extend(Z_PROJECTILE),
                    rotation,
                    ..Default::default()
                },
                ..Default::default()
            },
            marker_projectile: MarkerProjectile,
            marker_rocket:  MarkerRocket{ exhaust: Entity::PLACEHOLDER },
            rocket_target: RocketTarget(target_wisp),
        };
        Self { exhaust, base }
    }
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let Self { exhaust, mut base } = self;
        let exhaust_entity = commands.spawn(exhaust).id();

        base.marker_rocket.exhaust = exhaust_entity;
        commands.spawn(base).add_child(exhaust_entity).id()
    }
}

pub fn rocket_move_system(
    mut rockets: Query<(&mut Transform, &mut RocketTarget), With<MarkerRocket>>,
    time: Res<Time>,
    wisps: Query<(Entity, &Transform), (With<Wisp>, Without<MarkerRocket>)>,
) {
    let mut wisps_iter = wisps.iter();
    for (mut transform, mut target) in rockets.iter_mut() {
        let target_position = if let Ok((_, wisp_transform)) = wisps.get(*target.0) {
            wisp_transform.translation.xy()
        } else {
            wisps_iter.next().map_or(Vec2::ZERO, |(wisp_entity, wisp_transform)| {
                target.0 = wisp_entity.into();
                wisp_transform.translation.xy()
            })
        };

        // Calculate the direction vector to the target
        let direction_vector = (target_position - transform.translation.xy()).normalize();

        // Calculate the current forward direction (assuming it's the local y-axis)
        let current_direction = transform.local_x().xy();

        // Move the entity forward (along the local y-axis)
        transform.translation += (current_direction * time.delta_seconds() * 200.0).extend(0.0);

        // Calculate the target angle
        let target_angle = direction_vector.y.atan2(direction_vector.x);
        let current_angle = current_direction.y.atan2(current_direction.x);

        // Calculate the shortest rotation to the target angle
        let mut angle_diff = target_angle - current_angle;
        if angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        } else if angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }

        // Apply the rotation smoothly
        let rotation_speed = 1.5; // radians per second
        let max_rotation_speed = rotation_speed * time.delta_seconds();
        let rotation_amount = angle_diff.clamp(-max_rotation_speed, max_rotation_speed);
        transform.rotate(Quat::from_rotation_z(rotation_amount));

    }
}

pub fn rocket_hit_system(
    mut commands: Commands,
    rockets: Query<(Entity, &Transform, &RocketTarget), (With<MarkerRocket>, Without<Wisp>)>,
    wisps_grid: Res<WispsGrid>,
    wisps_transforms: Query<&Transform, (With<Wisp>, Without<MarkerRocket>)>,
    mut wisps_health: Query<&mut Health, With<Wisp>>,
) {
    for (entity, rocket_transform, target) in rockets.iter() {
        let rocket_coords = GridCoords::from_transform(&rocket_transform);
        if !rocket_coords.is_in_bounds(wisps_grid.bounds()) {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let Ok(wisp_transform) = wisps_transforms.get(*target.0) else { continue };
        if rocket_transform.translation.xy().distance(wisp_transform.translation.xy()) > 6. { continue; }

        let coords = GridCoords::from_transform(&rocket_transform);
        for (dx, dy) in ALL_DIRECTIONS.iter().chain(&[(0, 0)]) {
            let blast_zone_coords = coords.shifted((*dx, *dy));
            if !blast_zone_coords.is_in_bounds(wisps_grid.bounds()) { continue; }

            BuilderExplosion::new(blast_zone_coords).spawn(&mut commands);

            let wisps_in_coords = &wisps_grid[blast_zone_coords];
            for wisp in wisps_in_coords {
                let Ok(mut health) = wisps_health.get_mut(**wisp) else { continue }; // May not find wisp if the wisp spawned at the same frame.
                health.decrease(50);
            }
        }
        commands.entity(entity).despawn_recursive();
    }
}

pub fn exhaust_blinking_system(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &MarkerRocketExhaust)>,
) {
    for (mut sprite, _) in query.iter_mut() {
        sprite.color.set_alpha(if time.elapsed_seconds() % 1. < 0.85 { 1. } else { 0.0 });
    }
}