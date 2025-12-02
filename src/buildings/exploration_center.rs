use crate::prelude::*;
use crate::ui::indicators::{IndicatorDisplay, IndicatorType, Indicators};
use crate::map_objects::common::{ExpeditionTargetMarker, ExpeditionZone};
use crate::units::expedition_drone::BuilderExpeditionDrone;

pub struct ExplorationCenterPlugin;
impl Plugin for ExplorationCenterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                create_expedition_system.run_if(in_state(GameState::Running)),
            ))
            .add_observer(BuilderExplorationCenter::on_add)
            .register_db_loader::<BuilderExplorationCenter>(MapLoadingStage2::SpawnMapElements)
            .register_db_saver(BuilderExplorationCenter::on_game_save);
    }
}

pub const EXPLORATION_CENTER_BASE_IMAGE: &str = "buildings/exploration_center.png";


#[derive(Component)]
pub struct ExplorationCenterNewExpeditionTimer(pub Timer);

#[derive(Clone, Copy, Debug)]
pub struct ExplorationCenterSaveData {
    pub entity: Entity,
    pub health: f32,
}

#[derive(Component, SSS)]
pub struct BuilderExplorationCenter {
    pub grid_position: GridCoords,
    pub save_data: Option<ExplorationCenterSaveData>,
}
impl Saveable for BuilderExplorationCenter {
    fn save(self, tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
        let save_data = self.save_data.expect("BuilderExplorationCenter for saving purpose must have save_data");
        let entity_index = save_data.entity.index() as i64;

        tx.save_marker("exploration_centers", entity_index)?;
        tx.save_grid_coords(entity_index, self.grid_position)?;
        tx.save_health(entity_index, save_data.health)?;
        Ok(())
    }
}
impl Loadable for BuilderExplorationCenter {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult> {
        // Even if we don't strictly need pagination for ExplorationCenter (few items), respecting it is safer for generic loader logic
        let mut stmt = ctx.conn.prepare("SELECT id FROM exploration_centers LIMIT ?1 OFFSET ?2")?;
        let mut rows = stmt.query(ctx.pagination.as_params())?;
        
        let mut count = 0;
        while let Some(row) = rows.next()? {
            let old_id: i64 = row.get(0)?;
            let grid_position = ctx.conn.get_grid_coords(old_id)?;
            let health = ctx.conn.get_health(old_id)?;
            
            if let Some(new_entity) = ctx.get_new_entity_for_old(old_id) {
                let save_data = ExplorationCenterSaveData { entity: new_entity, health };
                ctx.commands.entity(new_entity).insert(BuilderExplorationCenter::new_for_saving(grid_position, save_data));
            } else {
                eprintln!("Warning: ExplorationCenter with old ID {} has no corresponding new entity", old_id);
            }
            count += 1;
        }

        Ok(count.into())
    }
}
impl BuilderExplorationCenter {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position, save_data: None }
    }
    pub fn new_for_saving(grid_position: GridCoords, save_data: ExplorationCenterSaveData) -> Self {
        Self { grid_position, save_data: Some(save_data) }
    }

    fn on_game_save(
        mut commands: Commands,
        exploration_centers: Query<(Entity, &GridCoords, &Health), With<ExplorationCenter>>,
    ) {
        if exploration_centers.is_empty() { return; }
        println!("Creating batch of BuilderExplorationCenter for saving. {} items", exploration_centers.iter().count());
        let batch = exploration_centers.iter().map(|(entity, coords, health)| {
            let save_data = ExplorationCenterSaveData {
                entity,
                health: health.get_current(),
            };
            BuilderExplorationCenter::new_for_saving(*coords, save_data)
        }).collect::<SaveableBatchCommand<_>>();
        commands.queue(batch);
    }

    pub fn on_add(
        trigger: On<Add, BuilderExplorationCenter>,
        mut commands: Commands,
        builders: Query<&BuilderExplorationCenter>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        let entity = trigger.entity;
        let Ok(builder) = builders.get(entity) else { return; };
        
        let building_info = almanach.get_building_info(BuildingType::ExplorationCenter);
        let grid_imprint = building_info.grid_imprint;
        
        let mut entity_commands = commands.entity(entity);
        if let Some(save_data) = &builder.save_data {
            // Save data
            entity_commands.insert(Health::new(save_data.health));
        }

        entity_commands
            .remove::<BuilderExplorationCenter>()
            .insert((
                ExplorationCenter,
                Sprite {
                    image: asset_server.load(EXPLORATION_CENTER_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                builder.grid_position,
                grid_imprint,
                NeedsPower::default(),
                ExplorationCenterNewExpeditionTimer(Timer::from_seconds(3.0, TimerMode::Repeating)),
                related![Modifiers[
                    (ModifierMaxHealth::from_baseline(building_info), ModifierSourceBaseline),
                ]],
                related![Indicators[
                    IndicatorType::NoPower,
                    IndicatorType::DisabledByPlayer,
                ]],
                children![
                    IndicatorDisplay::default(),
                ],
            ));
    }
}

pub fn create_expedition_system(
    mut commands: Commands,
    mut exploration_centres: Query<(&mut ExplorationCenterNewExpeditionTimer, &Transform), (With<ExplorationCenter>, With<HasPower>, Without<DisabledByPlayer>)>,
    expedition_zones: Query<(Entity, &Transform), (With<ExpeditionZone>, With<ExpeditionTargetMarker>)>,
    time: Res<Time>,
) {
    if expedition_zones.is_empty() { return; }
    let mut zones_positions = None;
    for (mut timer, exploration_center_transform) in exploration_centres.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if zones_positions.is_none() {
                // Cache to avoid recomputing zone positions for every exploration center
                zones_positions = Some(expedition_zones.iter().map(|(entity, transform)| (entity, transform.translation.xy())).collect::<Vec<_>>());
            }
            let center_position = exploration_center_transform.translation.xy();
            let closest_zone = zones_positions.as_ref().unwrap().iter().min_by(|a, b| {
                a.1.distance_squared(center_position).total_cmp(&b.1.distance_squared(center_position))
            });
            if let Some((zone_entity, ..)) = closest_zone {
                commands.spawn(BuilderExpeditionDrone::new(center_position, *zone_entity));
            }
        }
    }
}
