use crate::prelude::*;
use crate::map_objects::common::ExpeditionZone;

pub struct ExpeditionDronePlugin;
impl Plugin for ExpeditionDronePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                move_expedition_drone_system.run_if(in_state(GameState::Running)),
            ))
            .add_observer(BuilderExpeditionDrone::on_add);
    }
}

pub const EXPEDITION_DRONE_BASE_IMAGE: &str = "units/expedition_drone.png";

#[derive(Component)]
#[require(MapBound)]
pub struct ExpeditionDrone {
    target: Entity, // Entity having ExpeditionZone component
    target_world_position: Vec2,
}

#[derive(Component)]
pub struct BuilderExpeditionDrone {
    world_position: Vec2,
    target_entity: Entity,
}
impl BuilderExpeditionDrone {
    pub fn new(world_position: Vec2, target_entity: Entity) -> Self {
        Self { world_position, target_entity }
    }

    fn on_add(
        trigger: Trigger<OnAdd, BuilderExpeditionDrone>,
        mut commands: Commands,
        builders: Query<&BuilderExpeditionDrone>,
        asset_server: Res<AssetServer>,
        expedition_zones: Query<&Transform, With<ExpeditionZone>>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        if let Ok(exploration_zone_transform) = expedition_zones.get(builder.target_entity) {
            let target_vector = exploration_zone_transform.translation.xy() - builder.world_position;
            commands.entity(entity)
                .remove::<BuilderExpeditionDrone>()
                .insert((
                    Sprite {
                        image: asset_server.load(EXPEDITION_DRONE_BASE_IMAGE),
                        ..default()
                    },
                    Transform {
                        translation: builder.world_position.extend(Z_AERIAL_UNIT),
                        rotation: Quat::from_rotation_z(target_vector.y.atan2(target_vector.x)),
                        ..default()
                    },
                    ExpeditionDrone { target: builder.target_entity, target_world_position: exploration_zone_transform.translation.xy() },
                ));
        } else {
            // Exploration zone was removed; destroy the drone
            commands.entity(entity).despawn();
        }
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
        transform.translation += (target_vector.normalize() * time.delta_secs() * 100.0).extend(0.);
    }
}