use crate::prelude::*;
use lib_grid::grids::emissions::{EmissionsType, EmitterEnergy};
use lib_grid::grids::energy_supply::SupplierEnergy;
use lib_grid::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode};

pub struct EnergyRelayPlugin;
impl Plugin for EnergyRelayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderEnergyRelay>()
            .add_systems(PostUpdate, (
                BuilderEnergyRelay::spawn_system,
            ));
    }
}

pub const ENERGY_RELAY_BASE_IMAGE: &str = "buildings/energy_relay.png";

#[derive(Event)]
pub struct BuilderEnergyRelay {
    pub entity: Entity,
    pub grid_position: GridCoords,
}
impl BuilderEnergyRelay {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self {
        Self { entity, grid_position}
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderEnergyRelay>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        for &BuilderEnergyRelay{ entity, grid_position } in events.read() {
            let grid_imprint = almanach.get_building_info(BuildingType::EnergyRelay).grid_imprint;
            commands.entity(entity).insert((
                Sprite {
                    image: asset_server.load(ENERGY_RELAY_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    color: Color::hsla(0., 0.2, 1.0, 1.0), // 1.6 is a good value if the pulsation is off.
                    ..Default::default()
                },
                Transform::from_translation(grid_position.to_world_position_centered(grid_imprint).extend(Z_BUILDING)),
                grid_position,
                Health::new(100),
                Building,
                BuildingType::EnergyRelay,
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
}
impl Command for BuilderEnergyRelay {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}
