use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::common_components::{Health, TargetVector};
use crate::grids::common::GridCoords;
use crate::grids::wisps::WispsGrid;
use crate::projectiles::components::MarkerProjectile;
use crate::wisps::components::Wisp;

#[derive(Component)]
pub struct MarkerLaserDart;

pub fn create_laser_dart(commands: &mut Commands, world_position: Vec3, target_vector: TargetVector) -> Entity {
    let entity = commands.spawn(
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(7.0, 1.0)),
                ..Default::default()
            },
            transform: Transform {
                translation: world_position,
                rotation: Quat::from_rotation_z(target_vector.0.y.atan2(target_vector.0.x)),
                ..Default::default()
            },
            ..Default::default()
        }
    ).insert(
        (MarkerProjectile, MarkerLaserDart)
    ).insert(
        target_vector
    ).id();
    entity
}

pub fn laser_dart_move_system(
    mut laser_darts: Query<(&mut Transform, &TargetVector), With<MarkerLaserDart>>,
    time: Res<Time>,
) {
    for (mut transform, target_vector) in laser_darts.iter_mut() {
        transform.translation += target_vector.0.extend(0.) * time.delta_seconds() * 100.;
    }
}

pub fn laser_dart_hit_system(
    mut commands: Commands,
    laser_darts: Query<(Entity, &Transform), With<MarkerLaserDart>>,
    wisps_grid: Res<WispsGrid>,
    mut wisps: Query<(&mut Health, &Transform), With<Wisp>>,
) {
    for (entity, laser_dart_transform) in laser_darts.iter() {
        let coords = GridCoords::from_transform(&laser_dart_transform);
        if !coords.is_in_bounds(wisps_grid.bounds()) {
            commands.entity(entity).despawn();
            continue;
        }
        let wisps_in_coords = &wisps_grid[coords];
        for wisp in wisps_in_coords {
            let Ok((mut health, wisp_transform)) = wisps.get_mut(*wisp) else { continue }; // May not find wisp if the wisp spawned at the same frame.
            if laser_dart_transform.translation.xy().distance(wisp_transform.translation.xy()) < 8. {
                health.decrease(1);
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}