use bevy::prelude::*;
use crate::common::Z_AERIAL_UNIT;
use crate::map_objects::common::ExpeditionZone;

#[derive(Component)]
pub struct ExpeditionDrone {
    target: Entity, // Entity having ExpeditionZone component
    target_world_position: Vec2,
}

#[derive(Default)]
pub struct BundleExpeditionDrone {
    pub sprite: SpriteBundle,
    pub expedition_drone: Option<ExpeditionDrone>,
}

impl BundleExpeditionDrone {
    pub fn new(world_position: Vec2, asset_server: &AssetServer) -> Self {
        BundleExpeditionDrone {
            sprite: SpriteBundle {
                texture: asset_server.load("units/expedition_drone.png"),
                transform: Transform::from_translation(world_position.extend(Z_AERIAL_UNIT)),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    pub fn with_target(mut self, target: Entity, target_world_position: Vec2) -> Self {
        self.expedition_drone = Some(ExpeditionDrone { target, target_world_position });
        let target_vector = target_world_position - self.sprite.transform.translation.xy();
        self.sprite.transform.rotation = Quat::from_rotation_z(target_vector.y.atan2(target_vector.x));
        self
    }
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands
            .spawn(self.sprite)
            .insert(self.expedition_drone.expect("ExpeditionDrone must have ExpeditionDrone component"))
            .id()
    }
}

pub fn move_expedition_drone_system(
    mut commands: Commands,
    mut drones: Query<(Entity, &mut Transform, &ExpeditionDrone), Without<ExpeditionZone>>,
    mut zones: Query<(Entity, &Transform, &mut ExpeditionZone)>,
    time: Res<Time>,
) {
    for (entity, mut transform, drone) in drones.iter_mut() {
        let target_vector = drone.target_world_position - transform.translation.xy();
        let target_distance_squared = target_vector.length_squared();
        if target_distance_squared < 4. {
            if let Ok((_, _, mut zone)) = zones.get_mut(drone.target) {
                zone.expeditions_arrived += 1;
            }
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += (target_vector.normalize() * time.delta_seconds() * 100.0).extend(0.);
    }
}