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
        trigger: On<Add, BuilderExpeditionDrone>,
        mut commands: Commands,
        builders: Query<&BuilderExpeditionDrone>,
        asset_server: Res<AssetServer>,
    ) {
        let entity = trigger.entity;
        let Ok(builder) = builders.get(entity) else { return; };
        
        commands.entity(entity)
            .remove::<BuilderExpeditionDrone>()
            .insert((
                Sprite {
                    image: asset_server.load(EXPEDITION_DRONE_BASE_IMAGE),
                    ..default()
                },
                Transform {
                    translation: builder.world_position.extend(Z_AERIAL_UNIT),
                    scale: Vec3::new(2., 2., 1.),
                    ..default()
                },
                ExpeditionDrone { target: builder.target_entity },
            ));
    }
}

pub fn move_expedition_drone_system(
    mut commands: Commands,
    mut drones: Query<(Entity, &mut Transform, &ExpeditionDrone), Without<ExpeditionZone>>,
    mut zones: Query<(&Transform, &mut ExpeditionZone)>,
    time: Res<Time>,
) {
    for (entity, mut transform, drone) in drones.iter_mut() {
        if let Ok((target_transform, mut zone)) = zones.get_mut(drone.target) {
            let target_vector = target_transform.translation.xy() - transform.translation.xy();
            let target_distance_squared = target_vector.length_squared();
            
            if target_distance_squared < 4. {
                zone.expeditions_arrived += 1;
                commands.entity(entity).despawn();
                continue;
            }

            // Update rotation to face target
            transform.rotation = Quat::from_rotation_z(target_vector.y.atan2(target_vector.x));
            transform.translation += (target_vector.normalize() * time.delta_secs() * 200.0).extend(0.);
        } else {
            // Target zone no longer exists
            commands.entity(entity).despawn();
        }
    }
}