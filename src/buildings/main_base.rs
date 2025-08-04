use lib_grid::grids::emissions::{EmissionsType, EmitterEnergy};
use lib_grid::grids::energy_supply::{GeneratorEnergy, SupplierEnergy};
use lib_grid::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode};

use crate::prelude::*;


pub struct MainBasePlugin;
impl Plugin for MainBasePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderMainBase::on_add);
    }
}

pub const MAIN_BASE_BASE_IMAGE: &str = "buildings/main_base.png";


#[derive(Component)]
pub struct BuilderMainBase {
    grid_position: GridCoords,
}
impl BuilderMainBase {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
     }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderMainBase>,
        mut commands: Commands,
        builders: Query<&BuilderMainBase>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        let entity = trigger.target();
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
                TechnicalState { has_energy_supply: true, ..default() },
                related![Modifiers[
                    (ModifierMaxHealth(building_info.baseline[&ModifierType::MaxHealth]), ModifierSourceBaseline),
                    (ModifierEnergySupplyRange(building_info.baseline[&ModifierType::EnergySupplyRange]), ModifierSourceBaseline),
                ]],
            ));
    }
}