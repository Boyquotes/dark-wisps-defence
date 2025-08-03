use lib_grid::grids::energy_supply::EnergySupplyGrid;

use crate::effects::ripple::BuilderRipple;
use crate::prelude::*;
use crate::wisps::components::Wisp;

pub struct TowerEmitterPlugin;
impl Plugin for TowerEmitterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderTowerEmitter::on_add).add_systems(Update, (
                shooting_system.run_if(in_state(GameState::Running)),
            ));
    }
}

pub const TOWER_EMITTER_BASE_IMAGE: &str = "buildings/tower_emitter.png";

#[derive(Component)]
pub struct BuilderTowerEmitter {
    grid_position: GridCoords,
}

impl BuilderTowerEmitter {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderTowerEmitter>,
        mut commands: Commands,
        builders: Query<&BuilderTowerEmitter>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let building_info = almanach.get_building_info(BuildingType::Tower(TowerType::Emitter));
        let grid_imprint = building_info.grid_imprint;
        commands.entity(entity)
            .remove::<BuilderTowerEmitter>()
            .insert((
                Sprite {
                    image: asset_server.load(TOWER_EMITTER_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                MarkerTower,
                TowerEmitter,
                builder.grid_position,
                grid_imprint,
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(builder.grid_position, grid_imprint), ..default() },
                related![Modifiers[
                    (ModifierAttackRange(building_info.baseline[&ModifierType::AttackRange] as usize), ModifierSourceBaseline),
                    (ModifierAttackSpeed(building_info.baseline[&ModifierType::AttackSpeed]), ModifierSourceBaseline),
                    (ModifierAttackDamage(building_info.baseline[&ModifierType::AttackDamage] as i32), ModifierSourceBaseline),
                    (ModifierMaxHealth(building_info.baseline[&ModifierType::MaxHealth] as i32), ModifierSourceBaseline),
                ]],
            ));
        commands.trigger_targets(lib_inventory::almanach::AlmanachRequestPotentialUpgradesInsertion, entity);
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_emitters: Query<(&Transform, &TechnicalState, &AttackRange, &mut TowerShootingTimer, &mut TowerWispTarget), With<TowerEmitter>>,
    wisps: Query<(), With<Wisp>>,
) {
    for (transform, technical_state, range, mut timer, mut target) in tower_emitters.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        if !wisps.contains(target_wisp) {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        commands.spawn(BuilderRipple::new(transform.translation.xy(), range.0 as f32 * CELL_SIZE));
        timer.0.reset();
    }
}
