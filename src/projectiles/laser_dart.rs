use lib_grid::grids::wisps::WispsGrid;

use crate::prelude::*;
use crate::projectiles::components::Projectile;
use crate::wisps::components::Wisp;

pub struct LaserDartPlugin;
impl Plugin for LaserDartPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                (
                    laser_dart_move_system,
                    laser_dart_hit_system,
                ).run_if(in_state(GameState::Running)),
            ))
            .add_observer(BuilderLaserDart::on_add);
    }
}

#[derive(Component)]
pub struct LaserDart;

// LaserDart follows Wisp, and if the wisp no longer exists, follows the target vector
#[derive(Component, Default)]
#[require(AttackDamage, Projectile)]
pub struct LaserDartTarget {
    pub target_wisp: Option<Entity>,
    pub target_vector: Vec2,
}

#[derive(Component)]
pub struct BuilderLaserDart {
    world_position: Vec2,
    target_wisp: Entity,
    target_vector: Vec2,
    damage: AttackDamage,
}

impl BuilderLaserDart {
    pub fn new(world_position: Vec2, target_wisp: Entity, target_vector: Vec2, damage: AttackDamage) -> Self {
        Self { world_position, target_wisp, target_vector, damage }
    }

    fn on_add(
        trigger: On<Add, BuilderLaserDart>,
        mut commands: Commands,
        builders: Query<&BuilderLaserDart>,
    ) {
        let entity = trigger.entity;
        let Ok(builder) = builders.get(entity) else { return; };
        
        commands.entity(entity)
            .remove::<BuilderLaserDart>()
            .insert((
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(14.0, 2.0)),
                    ..Default::default()
                },
                Transform {
                    translation: builder.world_position.extend(Z_PROJECTILE),
                    rotation: Quat::from_rotation_z(builder.target_vector.y.atan2(builder.target_vector.x)),
                    ..Default::default()
                },
                LaserDart,
                LaserDartTarget{ target_wisp: Some(builder.target_wisp), target_vector: builder.target_vector },
                builder.damage.clone(),
            ));
    }
}

pub fn laser_dart_move_system(
    mut laser_darts: Query<(&mut Transform, &mut LaserDartTarget), With<LaserDart>>,
    wisps: Query<&Transform, (With<Wisp>, Without<LaserDart>)>,
    time: Res<Time>,
) {
    for (mut transform, mut target) in laser_darts.iter_mut() {
        // If the target wisp still exists - follow it by updating the target vector
        if let Some(target_wisp) = target.target_wisp {
            if let Ok(wisp_transform) = wisps.get(target_wisp) {
                target.target_vector = (wisp_transform.translation.xy() - transform.translation.xy()).normalize();
            } else {
                target.target_wisp = None;
            }
        }
        transform.translation += target.target_vector.extend(0.) * time.delta_secs() * 600.;
    }
}

pub fn laser_dart_hit_system(
    mut commands: Commands,
    laser_darts: Query<(Entity, &Transform, &AttackDamage), With<LaserDart>>,
    wisps_grid: Res<WispsGrid>,
    mut wisps: Query<(&mut Health, &Transform), With<Wisp>>,
) {
    for (entity, laser_dart_transform, damage) in laser_darts.iter() {
        let coords = GridCoords::from_transform(&laser_dart_transform);
        if !coords.is_in_bounds(wisps_grid.bounds()) {
            commands.entity(entity).despawn();
            continue;
        }
        let wisps_in_coords = &wisps_grid[coords];
        for wisp in wisps_in_coords {
            let Ok((mut health, wisp_transform)) = wisps.get_mut(*wisp) else { continue }; // May not find wisp if the wisp spawned at the same frame.
            if laser_dart_transform.translation.xy().distance(wisp_transform.translation.xy()) < 8. {
                health.decrease(damage.0);
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}