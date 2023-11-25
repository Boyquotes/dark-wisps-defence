use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, MarkerEnergyRelay, TechnicalState};
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::emissions::{EmissionsType, EmitterCreatedEvent, EmitterEnergy};
use crate::grids::energy_supply::{SupplierCreatedEvent, SupplierEnergy};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator};

pub const ENERGY_RELAY_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

pub fn create_energy_relay(
    commands: &mut Commands,
    emitter_created_event_writer: &mut EventWriter<EmitterCreatedEvent>,
    supplier_created_event_writer: &mut EventWriter<SupplierCreatedEvent>,
    obstacles_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) -> Entity {
    let building = Building {
        grid_imprint: ENERGY_RELAY_GRID_IMPRINT,
        building_type: BuildingType::EnergyRelay
    };
    let energy_emissions_details = FloodEmissionsDetails {
        emissions_type: EmissionsType::Energy,
        range: usize::MAX,
        evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
    };
    let supplier_energy = SupplierEnergy { range: 15 };
    let building_entity = commands.spawn((
        get_energy_relay_sprite_bundle(grid_position),
        MarkerEnergyRelay,
        grid_position,
        Health(10000),
        building.clone(),
        EmitterEnergy(energy_emissions_details.clone()),
        supplier_energy,
        TechnicalState{ has_energy_supply: true },
    )).id();
    emitter_created_event_writer.send(EmitterCreatedEvent {
        coords: building.grid_imprint.covered_coords(grid_position),
        emissions_details: vec![energy_emissions_details],
    });
    supplier_created_event_writer.send(SupplierCreatedEvent {
        coords: building.grid_imprint.covered_coords(grid_position),
        supplier: supplier_energy,
    });
    obstacles_grid.imprint(grid_position, Field::Building(building_entity, building.building_type), ENERGY_RELAY_GRID_IMPRINT);
    building_entity
}

pub fn get_energy_relay_sprite_bundle(coords: GridCoords) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(ENERGY_RELAY_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        transform: Transform::from_translation(coords.to_world_position_centered(ENERGY_RELAY_GRID_IMPRINT).extend(0.)),
        ..Default::default()
    }
}