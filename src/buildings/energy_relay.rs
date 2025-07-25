use crate::prelude::*;
use lib_grid::grids::emissions::{EmissionsType, EmitterEnergy};
use lib_grid::grids::energy_supply::SupplierEnergy;
use lib_grid::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode};

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
        
        let grid_imprint = almanach.get_building_info(BuildingType::EnergyRelay).grid_imprint;
        commands.entity(entity)
            .remove::<BuilderEnergyRelay>()
            .insert((
                Sprite {
                    image: asset_server.load(ENERGY_RELAY_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    color: Color::hsla(0., 0.2, 1.0, 1.0), // 1.6 is a good value if the pulsation is off.
                    ..Default::default()
                },
                builder.grid_position,
                Health::new(100),
                EnergyRelay,
                grid_imprint,
                EmitterEnergy(FloodEmissionsDetails {
                    emissions_type: EmissionsType::Energy,
                    range: usize::MAX,
                    evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
                    mode: FloodEmissionsMode::Increase,
                }),
                SupplierEnergy{ range: 15 },
                TechnicalState{ has_energy_supply: true, ..default() },
                ColorPulsation::new(1.0, 1.8, 3.0),
            ));
    }
}
