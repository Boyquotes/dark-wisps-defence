use crate::prelude::*;
use crate::grids::emissions::{EmissionsType, EmitterEnergy};
use crate::grids::energy_supply::SupplierEnergy;
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode};

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
            let grid_imprint = almanach.get_building_grid_imprint(BuildingType::EnergyRelay);
            commands.entity(entity).insert((
                get_energy_relay_sprite_bundle(grid_position, grid_imprint, &asset_server),
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

pub fn get_energy_relay_sprite_bundle(coords: GridCoords, grid_imprint: GridImprint, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            image: asset_server.load(ENERGY_RELAY_BASE_IMAGE),
            custom_size: Some(grid_imprint.world_size()),
            color: Color::hsla(0., 0.2, 1.0, 1.0), // 1.6 is a good value if the pulsation is off.
            ..Default::default()
        },
        transform: Transform::from_translation(coords.to_world_position_centered(grid_imprint).extend(Z_BUILDING)),
        ..Default::default()
    }
}