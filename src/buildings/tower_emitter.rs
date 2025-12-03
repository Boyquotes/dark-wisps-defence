use crate::effects::ripple::BuilderRipple;
use crate::prelude::*;
use crate::ui::indicators::{IndicatorDisplay, IndicatorType, Indicators};
use crate::wisps::components::Wisp;

pub struct TowerEmitterPlugin;
impl Plugin for TowerEmitterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderTowerEmitter::on_add).add_systems(Update, (
                shooting_system.run_if(in_state(GameState::Running)),
            ))
            .register_db_loader::<BuilderTowerEmitter>(MapLoadingStage2::SpawnMapElements)
            .register_db_saver(BuilderTowerEmitter::on_game_save);
    }
}

pub const TOWER_EMITTER_BASE_IMAGE: &str = "buildings/tower_emitter.png";

#[derive(Clone, Copy, Debug)]
pub struct TowerEmitterSaveData {
    entity: Entity,
    health: f32,
}

#[derive(Component, SSS)]
pub struct BuilderTowerEmitter {
    grid_position: GridCoords,
    save_data: Option<TowerEmitterSaveData>,
}

impl Saveable for BuilderTowerEmitter {
    fn save(self, tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
        let save_data = self.save_data.expect("BuilderTowerEmitter for saving must have save_data");
        let entity_index = save_data.entity.index() as i64;

        tx.save_marker("tower_emitters", entity_index)?;
        tx.save_grid_coords(entity_index, self.grid_position)?;
        tx.save_health(entity_index, save_data.health)?;
        Ok(())
    }
}

impl Loadable for BuilderTowerEmitter {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult> {
        let mut stmt = ctx.conn.prepare("SELECT id FROM tower_emitters LIMIT ?1 OFFSET ?2")?;
        let mut rows = stmt.query(ctx.pagination.as_params())?;
        
        let mut count = 0;
        while let Some(row) = rows.next()? {
            let old_id: i64 = row.get(0)?;
            let grid_position = ctx.conn.get_grid_coords(old_id)?;
            let health = ctx.conn.get_health(old_id)?;
            
            if let Some(new_entity) = ctx.get_new_entity_for_old(old_id) {
                let save_data = TowerEmitterSaveData { entity: new_entity, health };
                ctx.commands.entity(new_entity).insert(BuilderTowerEmitter::new_for_saving(grid_position, save_data));
            }
            count += 1;
        }

        Ok(count.into())
    }
}

impl BuilderTowerEmitter {
    pub fn new(grid_position: GridCoords) -> Self { Self { grid_position, save_data: None } }
    pub fn new_for_saving(grid_position: GridCoords, save_data: TowerEmitterSaveData) -> Self {
        Self { grid_position, save_data: Some(save_data) }
    }

    fn on_game_save(
        mut commands: Commands,
        towers: Query<(Entity, &GridCoords, &Health), With<TowerEmitter>>,
    ) {
        if towers.is_empty() { return; }
        let batch = towers.iter().map(|(entity, coords, health)| {
            let save_data = TowerEmitterSaveData {
                entity,
                health: health.get_current(),
            };
            BuilderTowerEmitter::new_for_saving(*coords, save_data)
        }).collect::<SaveableBatchCommand<_>>();
        commands.queue(batch);
    }

    pub fn on_add(
        trigger: On<Add, BuilderTowerEmitter>,
        mut commands: Commands,
        builders: Query<&BuilderTowerEmitter>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        let entity = trigger.entity;
        let Ok(builder) = builders.get(entity) else { return; };
        
        let building_info = almanach.get_building_info(BuildingType::Tower(TowerType::Emitter));
        let grid_imprint = building_info.grid_imprint;

        let mut entity_commands = commands.entity(entity);
        if let Some(save_data) = &builder.save_data {
            entity_commands.insert(Health::new(save_data.health));
        }

        entity_commands
            .remove::<BuilderTowerEmitter>()
            .insert((
                TowerEmitter,
                Sprite {
                    image: asset_server.load(TOWER_EMITTER_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                Tower,
                builder.grid_position,
                grid_imprint,
                NeedsPower::default(),
                related![Modifiers[
                    (ModifierAttackRange::from_baseline(building_info), ModifierSourceBaseline),
                    (ModifierAttackSpeed::from_baseline(building_info), ModifierSourceBaseline),
                    (ModifierAttackDamage::from_baseline(building_info), ModifierSourceBaseline),
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
        commands.trigger(lib_inventory::almanach::AlmanachRequestPotentialUpgradesInsertion { entity });
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_emitters: Query<(&Transform, &AttackRange, &mut TowerShootingTimer, &mut TowerWispTarget), (With<TowerEmitter>, With<HasPower>, Without<DisabledByPlayer>)>,
    wisps: Query<(), With<Wisp>>,
) {
    for (transform, range, mut timer, mut target) in tower_emitters.iter_mut() {
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.is_finished() { continue; }

        if !wisps.contains(target_wisp) {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        commands.spawn(BuilderRipple::new(transform.translation.xy(), range.0 as f32 * CELL_SIZE));
        timer.0.reset();
    }
}
