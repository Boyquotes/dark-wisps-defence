use lib_grid::grids::emissions::{EmissionsType, EmitterEnergy, EmitterEnergyEnabled};
use lib_grid::grids::energy_supply::SupplierEnergy;
use lib_grid::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode};

use crate::prelude::*;
use crate::ui::indicators::{IndicatorDisplay, IndicatorType, Indicators};

pub struct EnergyRelayPlugin;
impl Plugin for EnergyRelayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderEnergyRelay::on_add);
    }
}

pub const ENERGY_RELAY_BASE_IMAGE: &str = "buildings/energy_relay.png";

#[derive(Component)]
pub struct BuilderEnergyRelay {
    grid_position: GridCoords,
}
impl BuilderEnergyRelay {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderEnergyRelay>,
        mut commands: Commands,
        builders: Query<&BuilderEnergyRelay>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let building_info = almanach.get_building_info(BuildingType::EnergyRelay);
        commands.entity(entity)
            .remove::<BuilderEnergyRelay>()
            .insert((
                EnergyRelay,
                Sprite {
                    image: asset_server.load(ENERGY_RELAY_BASE_IMAGE),
                    custom_size: Some(building_info.grid_imprint.world_size()),
                    color: Color::hsla(0., 0.2, 1.0, 1.0), // 1.6 is a good value if the pulsation is off.
                    ..Default::default()
                },
                builder.grid_position,
                building_info.grid_imprint,
                NeedsPower::default(),
                EmitterEnergy(FloodEmissionsDetails {
                    emissions_type: EmissionsType::Energy,
                    range: usize::MAX,
                    evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
                    mode: FloodEmissionsMode::Increase,
                }),
                SupplierEnergy,
                related![Modifiers[
                    (ModifierMaxHealth::from_baseline(building_info), ModifierSourceBaseline),
                    (ModifierEnergySupplyRange::from_baseline(building_info), ModifierSourceBaseline),
                ]],
                related![Indicators[
                    IndicatorType::NoPower,
                    IndicatorType::DisabledByPlayer,
                ]],
                children![
                    IndicatorDisplay::default(),
                ],
            ))
            .observe(|trigger: Trigger<OnInsert, HasPower>, mut commands: Commands| { commands.trigger_targets(RequestTechnicalStateUpdate, trigger.target()); })
            .observe(|trigger: Trigger<OnInsert, NoPower>, mut commands: Commands| { commands.trigger_targets(RequestTechnicalStateUpdate, trigger.target()); })
            .observe(|trigger: Trigger<OnInsert, DisabledByPlayer>, mut commands: Commands| { commands.trigger_targets(RequestTechnicalStateUpdate, trigger.target()); })
            .observe(|trigger: Trigger<OnRemove, DisabledByPlayer>, mut commands: Commands| { commands.trigger_targets(RequestTechnicalStateUpdate, trigger.target()); })
            .observe(RequestTechnicalStateUpdate::on_trigger)
            ;

        commands.trigger_targets(RequestTechnicalStateUpdate, entity);
    }
}

#[derive(Event)]
struct RequestTechnicalStateUpdate;
impl RequestTechnicalStateUpdate {
    fn on_trigger(
        trigger: Trigger<RequestTechnicalStateUpdate>,
        mut commands: Commands,
        relays: Query<(Has<DisabledByPlayer>, Has<NoPower>), With<EnergyRelay>>,
    ) {
        let entity = trigger.target();
        let Ok((has_disabled_by_player, has_no_power)) = relays.get(entity) else { return; };
        let mut entity_commands = commands.entity(entity);
        if has_disabled_by_player {
            entity_commands.remove::<SupplierEnergy>().remove::<EmitterEnergyEnabled>().remove::<ColorPulsation>();
        }
        else if has_no_power {
            entity_commands.remove::<EmitterEnergyEnabled>().remove::<ColorPulsation>().try_insert(SupplierEnergy);
        } else {
            entity_commands.try_insert(SupplierEnergy).try_insert(EmitterEnergyEnabled).try_insert(ColorPulsation::new(1.0, 1.8, 3.0));
        }
    }
}