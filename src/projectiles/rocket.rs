use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::common_components::{Health};
use crate::grids::common::GridCoords;
use crate::grids::wisps::WispsGrid;
use crate::projectiles::components::MarkerProjectile;
use crate::search::common::ALL_DIRECTIONS;
use crate::wisps::components::{Wisp, WispEntity};

#[derive(Component)]
pub struct MarkerRocket;

// Rocket follows Wisp, and if the wisp no longer exists, looks for another target
#[derive(Component)]
pub struct RocketTarget(pub WispEntity);

pub fn create_rocket(commands: &mut Commands, world_position: Vec3, target_wisp: WispEntity) -> Entity {
    let entity = commands.spawn(
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(16.0, 6.0)),
                ..Default::default()
            },
            transform: Transform {
                translation: world_position,
                ..Default::default()
            },
            ..Default::default()
        }
    ).insert((
        MarkerProjectile,
        MarkerRocket,
        RocketTarget(target_wisp),
    )).id();
    entity
}

pub fn rocket_move_system(
    mut rockets: Query<(&mut Transform, &mut RocketTarget), With<MarkerRocket>>,
    time: Res<Time>,
    wisps: Query<(Entity, &Transform), (With<Wisp>, Without<MarkerRocket>)>,
) {
    for (mut transform, mut target) in rockets.iter_mut() {
        let target_position = if let Ok((_, wisp_transform)) = wisps.get(*target.0) {
            wisp_transform.translation.xy()
        } else {
            wisps.iter().next().map_or(Vec2::ZERO, |(wisp_entity, wisp_transform)| {
                target.0 = wisp_entity.into();
                wisp_transform.translation.xy()
            })
        };

        let direction_vector = (target_position - transform.translation.xy()).normalize();
        let move_distance = direction_vector * time.delta_seconds() * 200.;

        // Move the rocket
        transform.translation += move_distance.extend(0.);
        // Rotate the rocket
        transform.rotation = Quat::from_rotation_z(direction_vector.y.atan2(direction_vector.x));
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
            commands.entity(entity).despawn();
            continue;
        }

        let Ok(wisp_transform) = wisps_transforms.get(*target.0) else { continue };
        if rocket_transform.translation.xy().distance(wisp_transform.translation.xy()) > 1. { continue; }

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
        commands.entity(entity).despawn();
    }
}