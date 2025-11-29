use lib_core::states::MapLoadingStage2;
use lib_grid::grids::emissions::{EmissionsType, EmitterEnergy};
use lib_grid::grids::energy_supply::{GeneratorEnergy, SupplierEnergy};
use lib_grid::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode};

use crate::prelude::*;


pub struct MainBasePlugin;
impl Plugin for MainBasePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderMainBase::on_add)
            .register_db_loader::<BuilderMainBase>(MapLoadingStage2::SpawnMapElements)
            .register_db_saver(BuilderMainBase::on_game_save);
    }
}

pub const MAIN_BASE_BASE_IMAGE: &str = "buildings/main_base.png";


#[derive(Component, SSS)]
pub struct BuilderMainBase {
    pub grid_position: GridCoords,
    pub entity: Option<Entity>,
}
impl Saveable for BuilderMainBase {
    fn save(self, tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
        let entity_index = self.entity.expect("BuilderMainBase for saving purpose must have an entity").index() as i64;

        // 0. Ensure entity exists in entities table
        tx.execute(
            "INSERT OR IGNORE INTO entities (id) VALUES (?1)",
            [entity_index],
        )?;

        // 1. Insert into main_base table (which stores the entity info)
        tx.execute(
            "INSERT OR REPLACE INTO main_bases (id) VALUES (?1)",
            [entity_index],
        )?;

        // 2. Insert into grid_positions table
        tx.execute(
            "INSERT INTO grid_positions (entity_id, x, y) VALUES (?1, ?2, ?3)",
            (entity_index, self.grid_position.x, self.grid_position.y),
        )?;
        Ok(())
    }
}
impl Loadable for BuilderMainBase {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult> {
        // Singleton load - but we query by ID from the table now
        let mut stmt = ctx.conn.prepare(
            "SELECT mb.id, gp.x, gp.y FROM main_bases mb
             JOIN grid_positions gp ON mb.id = gp.entity_id"
        )?;
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            let old_id: u64 = row.get(0)?;
            let x: i32 = row.get(1)?;
            let y: i32 = row.get(2)?;
            let grid_position = GridCoords { x, y };
            
            if let Some(new_entity) = ctx.get_new_entity_for_old(old_id) {
                ctx.commands.entity(new_entity).insert(BuilderMainBase::new_for_saving(grid_position, new_entity));
            } else {
                eprintln!("Warning: MainBase with old ID {} has no corresponding new entity", old_id);
            }
        }

        Ok(LoadResult::Finished)
    }
}
impl BuilderMainBase {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position, entity: None }
    }
    pub fn new_for_saving(grid_position: GridCoords, entity: Entity) -> Self {
        Self { grid_position, entity: Some(entity) }
    }

    fn on_game_save(
        mut commands: Commands,
        main_base: Query<(Entity, &GridCoords), With<MainBase>>,
    ) {
        if let Ok((entity, coords)) = main_base.single() {
            commands.queue(SaveableBatchCommand::from_single(BuilderMainBase::new_for_saving(*coords, entity)));
        }
    }

    pub fn on_add(
        trigger: On<Add, BuilderMainBase>,
        mut commands: Commands,
        builders: Query<&BuilderMainBase>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        let entity = trigger.entity;
        let Ok(builder) = builders.get(entity) else { return; };

        let building_info = almanach.get_building_info(BuildingType::MainBase);
        let grid_imprint = building_info.grid_imprint;
        commands.entity(entity)
            .remove::<BuilderMainBase>()
            .insert((
                Sprite {
                    image: asset_server.load(MAIN_BASE_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                MainBase,
                builder.grid_position,
                grid_imprint,
                EmitterEnergy(FloodEmissionsDetails {
                    emissions_type: EmissionsType::Energy,
                    range: usize::MAX,
                    evaluator: FloodEmissionsEvaluator::ExponentialDecay { start_value: 100., decay: 0.1 },
                    mode: FloodEmissionsMode::Increase,
                }),
                GeneratorEnergy,
                SupplierEnergy,
                related![Modifiers[
                    (ModifierMaxHealth::from_baseline(building_info), ModifierSourceBaseline),
                    (ModifierEnergySupplyRange::from_baseline(building_info), ModifierSourceBaseline),
                ]],
            ));
    }
}