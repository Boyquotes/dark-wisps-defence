use crate::prelude::*;
use crate::common::Z_AERIAL_UNIT;
use crate::map_objects::common::ExpeditionZone;

pub struct ExpeditionDronePlugin;
impl Plugin for ExpeditionDronePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderExpeditionDrone>()
            .add_systems(PostUpdate, (
                BuilderExpeditionDrone::spawn_system,
            ))
            .add_systems(Update, (
                move_expedition_drone_system,
            ));
    }
}

pub const EXPEDITION_DRONE_BASE_IMAGE: &str = "units/expedition_drone.png";

#[derive(Component)]
pub struct ExpeditionDrone {
    target: Entity, // Entity having ExpeditionZone component
    target_world_position: Vec2,
}

#[derive(Event)]
pub struct BuilderExpeditionDrone {
    pub entity: LazyEntity,
    pub world_position: Vec2,
    pub target_entity: Entity,
}
impl BuilderExpeditionDrone {
    pub fn new(world_position: Vec2, target_entity: Entity) -> Self {
        Self { entity: LazyEntity::default(), world_position, target_entity }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderExpeditionDrone>,
        asset_server: Res<AssetServer>,
        expedition_zones: Query<&Transform, With<ExpeditionZone>>,
    ) {
        for &BuilderExpeditionDrone{ mut entity, world_position, target_entity } in events.read() {
            let entity = entity.get(&mut commands);
            if let Ok(exploration_zone_transform) = expedition_zones.get(target_entity) {
                let target_vector = exploration_zone_transform.translation.xy() - world_position;
                commands.entity(entity).insert((
                    SpriteBundle {
                        texture: asset_server.load(EXPEDITION_DRONE_BASE_IMAGE),
                        transform: Transform {
                            translation: world_position.extend(Z_AERIAL_UNIT),
                            rotation: Quat::from_rotation_z(target_vector.y.atan2(target_vector.x)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ExpeditionDrone { target: target_entity, target_world_position: exploration_zone_transform.translation.xy() },
                ));
            } else {
                // Exploration zone was removed; destroy the drone
                commands.entity(entity).despawn();
            }
        }
    }
}
impl Command for BuilderExpeditionDrone {
    fn apply(self, world: &mut World) {
        world.send_event(self);
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