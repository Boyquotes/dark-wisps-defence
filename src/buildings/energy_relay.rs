use crate::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, TechnicalState};
use crate::grids::emissions::{EmissionsType, EmitterChangedEvent, EmitterEnergy};
use crate::grids::energy_supply::{SupplierChangedEvent, SupplierEnergy};
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode, FloodEnergySupplyMode};

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


pub const ENERGY_RELAY_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 2, height: 2 };
pub const ENERGY_RELAY_BASE_IMAGE: &str = "buildings/energy_relay.png";

#[derive(Component)]
pub struct MarkerEnergyRelay;

#[derive(Event)]
pub struct BuilderEnergyRelay {
    pub entity: LazyEntity,
    pub grid_position: GridCoords,
}
impl BuilderEnergyRelay {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { entity: LazyEntity::default(), grid_position}
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderEnergyRelay>,
        asset_server: Res<AssetServer>,
        mut emitter_created_event_writer: EventWriter<EmitterChangedEvent>,
        mut supplier_created_event_writer: EventWriter<SupplierChangedEvent>,
    ) {
        for &BuilderEnergyRelay{ mut entity, grid_position } in events.read() {
            let emmision_details = FloodEmissionsDetails {
                emissions_type: EmissionsType::Energy,
                range: usize::MAX,
                evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
                mode: FloodEmissionsMode::Increase,
            };
            let supplier_energy = SupplierEnergy { range: 15 };
            let entity = entity.get(&mut commands); 
            commands.entity(entity).insert((
                get_energy_relay_sprite_bundle(grid_position, &asset_server),
                MarkerEnergyRelay,
                grid_position,
                Health(10000),
                Building::from(BuildingType::EnergyRelay),
                EmitterEnergy(emmision_details.clone()),
                supplier_energy.clone(),
                TechnicalState{ has_energy_supply: true },
                ColorPulsation::new(1.0, 1.8, 3.0),
            ));
            let covered_coords = ENERGY_RELAY_GRID_IMPRINT.covered_coords(grid_position);
            emitter_created_event_writer.send(EmitterChangedEvent {
                emitter_entity: entity,
                coords: covered_coords.clone(),
                emissions_details: vec![emmision_details],
            });
            supplier_created_event_writer.send(SupplierChangedEvent {
                coords: covered_coords,
                supplier: supplier_energy,
                mode: FloodEnergySupplyMode::Increase,
            });
        }
    }
}
impl Command for BuilderEnergyRelay {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn get_energy_relay_sprite_bundle(coords: GridCoords, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(ENERGY_RELAY_GRID_IMPRINT.world_size()),
            color: Color::hsla(0., 0.2, 1.0, 1.0), // 1.6 is a good value if the pulsation is off.
            ..Default::default()
        },
        texture: asset_server.load(ENERGY_RELAY_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(ENERGY_RELAY_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}