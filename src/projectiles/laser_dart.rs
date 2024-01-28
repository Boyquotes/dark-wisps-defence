use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::common::Z_PROJECTILE;
use crate::common_components::{Health};
use crate::grids::common::GridCoords;
use crate::grids::wisps::WispsGrid;
use crate::projectiles::components::MarkerProjectile;
use crate::wisps::components::{Wisp, WispEntity};

#[derive(Component)]
pub struct MarkerLaserDart;

// LaserDart follows Wisp, and if the wisp no longer exists, follows the target vector
#[derive(Component, Default)]
pub struct LaserDartTarget {
    pub target_wisp: Option<WispEntity>,
    pub target_vector: Vec2,
}

#[derive(Bundle)]
pub struct BundleLaserDart {
    pub sprite: SpriteBundle,
    pub marker_projectile: MarkerProjectile,
    pub marker_laser_dart: MarkerLaserDart,
    pub laser_dart_target: LaserDartTarget,
}
impl BundleLaserDart {
    pub fn new(world_position: Vec2, target_wisp: WispEntity, target_vector: Vec2) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(7.0, 1.0)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: world_position.extend(Z_PROJECTILE),
                    rotation: Quat::from_rotation_z(target_vector.y.atan2(target_vector.x)),
                    ..Default::default()
                },
                ..Default::default()
            },
            marker_projectile: MarkerProjectile,
            marker_laser_dart: MarkerLaserDart,
            laser_dart_target: LaserDartTarget{ target_wisp: Some(target_wisp), target_vector },
        }
    }
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands.spawn(self).id()
    }
}

pub fn laser_dart_move_system(
    mut laser_darts: Query<(&mut Transform, &mut LaserDartTarget), With<MarkerLaserDart>>,
    wisps: Query<&Transform, (With<Wisp>, Without<MarkerLaserDart>)>,
    time: Res<Time>,
) {
    for (mut transform, mut target) in laser_darts.iter_mut() {
        // If the target wisp still exists - follow it by updating the target vector
        if let Some(target_wisp) = target.target_wisp {
            if let Ok(wisp_transform) = wisps.get(*target_wisp) {
                target.target_vector = (wisp_transform.translation.xy() - transform.translation.xy()).normalize();
            } else {
                target.target_wisp = None;
            }
        }
        transform.translation += target.target_vector.extend(0.) * time.delta_seconds() * 300.;
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
            let Ok((mut health, wisp_transform)) = wisps.get_mut(**wisp) else { continue }; // May not find wisp if the wisp spawned at the same frame.
            if laser_dart_transform.translation.xy().distance(wisp_transform.translation.xy()) < 8. {
                health.decrease(1);
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}