use crate::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, TechnicalState};
use crate::grids::emissions::{EmissionsType, EmitterChangedEvent, EmitterEnergy};
use crate::grids::energy_supply::{SupplierChangedEvent, SupplierEnergy};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode, FloodEnergySupplyMode};

pub const ENERGY_RELAY_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 2, height: 2 };
pub const ENERGY_RELAY_BASE_IMAGE: &str = "buildings/energy_relay.png";

#[derive(Component)]
pub struct MarkerEnergyRelay;

#[derive(Bundle)]
pub struct BuilderEnergyRelay {
    pub sprite_bundle: SpriteBundle,
    pub marker_energy_relay: MarkerEnergyRelay,
    pub grid_position: GridCoords,
    pub health: Health,
    pub building: Building,
    pub emitter_energy: EmitterEnergy,
    pub supplier_energy: SupplierEnergy,
    pub technical_state: TechnicalState,
    pub color_pulsation: ColorPulsation,
}
impl BuilderEnergyRelay {
    pub fn new(grid_position: GridCoords, asset_server: &AssetServer) -> Self {
        Self {
            sprite_bundle: get_energy_relay_sprite_bundle(grid_position, asset_server),
            marker_energy_relay: MarkerEnergyRelay,
            grid_position,
            health: Health(10000),
            building: Building::from(BuildingType::EnergyRelay),
            emitter_energy: EmitterEnergy(FloodEmissionsDetails {
                emissions_type: EmissionsType::Energy,
                range: usize::MAX,
                evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
                mode: FloodEmissionsMode::Increase,
            }),
            supplier_energy: SupplierEnergy { range: 15 },
            technical_state: TechnicalState{ has_energy_supply: true },
            color_pulsation: ColorPulsation::new(1.0, 1.8, 3.0),
        }
    }
    pub fn spawn(
        self,
        commands: &mut Commands,
        emitter_created_event_writer: &mut EventWriter<EmitterChangedEvent>,
        supplier_created_event_writer: &mut EventWriter<SupplierChangedEvent>,
        obstacles_grid: &mut ObstacleGrid,
    ) -> Entity {
        let grid_position = self.grid_position;
        let covered_coords = ENERGY_RELAY_GRID_IMPRINT.covered_coords(grid_position);
        emitter_created_event_writer.send(EmitterChangedEvent {
            coords: covered_coords.clone(),
            emissions_details: vec![self.emitter_energy.0.clone()],
        });
        supplier_created_event_writer.send(SupplierChangedEvent {
            coords: covered_coords,
            supplier: self.supplier_energy.clone(),
            mode: FloodEnergySupplyMode::Increase,
        });

        let entity = commands.spawn(self).id();

        obstacles_grid.imprint(grid_position, Field::Building(entity, BuildingType::EnergyRelay), ENERGY_RELAY_GRID_IMPRINT);
        entity
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