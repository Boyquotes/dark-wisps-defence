use bevy::prelude::*;
use crate::common_components::TargetVector;
use crate::map_editor::MapInfo;
use crate::projectiles::components::MarkerProjectile;

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

pub fn laser_dart_extinguish_system(
    mut commands: Commands,
    laser_darts: Query<(Entity, &Transform), With<MarkerLaserDart>>,
    map_info: Res<MapInfo>,
) {
    for (entity, transform) in laser_darts.iter() {
        let (x, y) = (transform.translation.x, transform.translation.y);
        if x < 0. || x > map_info.world_width || y < 0. || y > map_info.world_height {
            commands.entity(entity).despawn();
        }
    }
}