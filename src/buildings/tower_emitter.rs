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
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let building_info = almanach.get_building_info(BuildingType::Tower(TowerType::Emitter));
        let grid_imprint = building_info.grid_imprint;
        commands.entity(entity)
            .remove::<BuilderTowerEmitter>()
            .insert((
                TowerEmitter,
                Sprite {
                    image: asset_server.load(TOWER_EMITTER_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                MarkerTower,
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
        commands.trigger_targets(lib_inventory::almanach::AlmanachRequestPotentialUpgradesInsertion, entity);
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_emitters: Query<(&Transform, &AttackRange, &mut TowerShootingTimer, &mut TowerWispTarget), (With<TowerEmitter>, With<HasPower>)>,
    wisps: Query<(), With<Wisp>>,
) {
    for (transform, range, mut timer, mut target) in tower_emitters.iter_mut() {
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
