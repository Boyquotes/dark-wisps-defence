use std::sync::OnceLock;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::common::{Z_PROJECTILE, Z_PROJECTILE_UNDER};
use crate::common_components::{Health};
use crate::grids::common::GridCoords;
use crate::grids::wisps::WispsGrid;
use crate::projectiles::components::MarkerProjectile;
use crate::search::common::ALL_DIRECTIONS;
use crate::utils::math::angle_difference;
use crate::wisps::components::{Wisp, WispEntity};

pub static ROCKET_BASE_IMAGE: OnceLock<Handle<Image>> = OnceLock::new();
pub static ROCKET_EXHAUST_IMAGE: OnceLock<Handle<Image>> = OnceLock::new();

#[derive(Component)]
pub struct MarkerRocket {
    pub exhaust: Entity,
}
#[derive(Component)]
pub struct MarkerRocketExhaust;

// Rocket follows Wisp, and if the wisp no longer exists, looks for another target
#[derive(Component)]
pub struct RocketTarget(pub WispEntity);

pub fn create_rocket(commands: &mut Commands, world_position: Vec2, rotation: Quat, target_wisp: WispEntity) -> Entity {
    let exhaust_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10.0, 6.25)),
                anchor: Anchor::Custom(Vec2::new(0.75, 0.)),
                ..Default::default()
            },
            texture: ROCKET_EXHAUST_IMAGE.get().unwrap().clone(),
            transform: Transform {
                translation: Vec2::ZERO.extend(Z_PROJECTILE_UNDER),
                ..Default::default()
            },
            ..Default::default()
        },
        MarkerRocketExhaust,
    )).id();
    let rocket_entity = commands.spawn(
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 10.0)),
                ..Default::default()
            },
            texture: ROCKET_BASE_IMAGE.get().unwrap().clone(),
            transform: Transform {
                translation: world_position.extend(Z_PROJECTILE),
                rotation,
                ..Default::default()
            },
            ..Default::default()
        }
    ).insert((
        MarkerProjectile,
        MarkerRocket{ exhaust: exhaust_entity },
        RocketTarget(target_wisp),
    )).id();
    commands.entity(rocket_entity).add_child(exhaust_entity);
    rocket_entity
}

pub fn load_assets_system(asset_server: Res<AssetServer>) {
    ROCKET_BASE_IMAGE.set(asset_server.load("projectiles/rocket.png")).unwrap();
    ROCKET_EXHAUST_IMAGE.set(asset_server.load("projectiles/rocket_exhaust.png")).unwrap();
}

pub fn rocket_move_system(
    mut rockets: Query<(&mut Transform, &mut RocketTarget, &MarkerRocket), Without<MarkerRocketExhaust>>,
    time: Res<Time>,
    wisps: Query<(Entity, &Transform), (With<Wisp>, Without<MarkerRocket>, Without<MarkerRocketExhaust>)>,
) {
    for (mut transform, mut target, rocket) in rockets.iter_mut() {
        let target_position = if let Ok((_, wisp_transform)) = wisps.get(*target.0) {
            wisp_transform.translation.xy()
        } else {
            wisps.iter().next().map_or(Vec2::ZERO, |(wisp_entity, wisp_transform)| {
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
        sprite.color.set_a(if time.elapsed_seconds() % 1. < 0.85 { 1. } else { 0.0 });
    }
}