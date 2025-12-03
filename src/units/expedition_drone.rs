use crate::prelude::*;
use crate::map_objects::common::ExpeditionZone;

pub struct ExpeditionDronePlugin;
impl Plugin for ExpeditionDronePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                move_expedition_drone_system.run_if(in_state(GameState::Running)),
            ))
            .add_observer(BuilderExpeditionDrone::on_add)
            .register_db_loader::<BuilderExpeditionDrone>(MapLoadingStage2::SpawnMapElements)
            .register_db_saver(BuilderExpeditionDrone::on_game_save);
    }
}

pub const EXPEDITION_DRONE_BASE_IMAGE: &str = "units/expedition_drone.png";

#[derive(Component)]
#[require(MapBound)]
pub struct ExpeditionDrone {
    target: Entity, // Entity having ExpeditionZone component
}

#[derive(Component, SSS)]
pub struct BuilderExpeditionDrone {
    world_position: Vec2,
    target_entity: Entity,
    entity: Option<Entity>,
}
impl Saveable for BuilderExpeditionDrone {
    fn save(self, tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
        let entity = self.entity.expect("BuilderExpeditionDrone for saving must have entity");
        let entity_id = entity.index() as i64;
        let target_id = self.target_entity.index() as i64;

        tx.register_entity(entity_id)?;
        tx.save_world_position(entity_id, self.world_position)?;
        tx.execute(
            "INSERT OR REPLACE INTO expedition_drones (id, target_id) VALUES (?1, ?2)",
            rusqlite::params![entity_id, target_id],
        )?;
        Ok(())
    }
}
impl Loadable for BuilderExpeditionDrone {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult> {
        let mut stmt = ctx.conn.prepare("SELECT id, target_id FROM expedition_drones LIMIT ?1 OFFSET ?2")?;
        let mut rows = stmt.query(ctx.pagination.as_params())?;
        
        let mut count = 0;
        while let Some(row) = rows.next()? {
            let old_id: i64 = row.get(0)?;
            let target_old_id: i64 = row.get(1)?;
            let world_position = ctx.conn.get_world_position(old_id)?;
            
            let Some(new_entity) = ctx.get_new_entity_for_old(old_id) else { continue; };
            let Some(new_target_entity) = ctx.get_new_entity_for_old(target_old_id) else { continue; }; 
            ctx.commands.entity(new_entity).insert(BuilderExpeditionDrone::new_for_saving(world_position,new_target_entity,new_entity));
            count += 1;
        }
        Ok(count.into())
    }
}
impl BuilderExpeditionDrone {
    pub fn new(world_position: Vec2, target_entity: Entity) -> Self {
        Self { world_position, target_entity, entity: None }
    }
    pub fn new_for_saving(world_position: Vec2, target_entity: Entity, entity: Entity) -> Self {
        Self { world_position, target_entity, entity: Some(entity) }
    }

    fn on_game_save(
        mut commands: Commands,
        drones: Query<(Entity, &Transform, &ExpeditionDrone)>,
    ) {
        if drones.is_empty() { return; }
        let batch = drones.iter().map(|(entity, transform, drone)| {
             BuilderExpeditionDrone::new_for_saving(
                 transform.translation.xy(),
                 drone.target,
                 entity
             )
        }).collect::<SaveableBatchCommand<_>>();
        commands.queue(batch);
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