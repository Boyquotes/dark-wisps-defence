use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, MarkerEnergyRelay, TechnicalState};
use crate::common_components::Health;
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::grids::emissions::{EmissionsType, EmitterCreatedEvent, EmitterEnergy};
use crate::grids::energy_supply::{SupplierCreatedEvent, SupplierEnergy};
use crate::grids::obstacles::ObstacleGrid;
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator};

const ENERGY_RELAY_GRID_WIDTH: i32 = 1;
const ENERGY_RELAY_GRID_HEIGHT: i32 = 1;
const ENERGY_RELAY_WORLD_WIDTH: f32 = CELL_SIZE * ENERGY_RELAY_GRID_WIDTH as f32;
const ENERGY_RELAY_WORLD_HEIGHT: f32 = CELL_SIZE * ENERGY_RELAY_GRID_HEIGHT as f32;

pub fn create_energy_relay(
    commands: &mut Commands,
    emitter_created_event_writer: &mut EventWriter<EmitterCreatedEvent>,
    supplier_created_event_writer: &mut EventWriter<SupplierCreatedEvent>,
    obstacles_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) -> Entity {
    let building = Building {
        grid_imprint: get_energy_relay_grid_imprint(),
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
    obstacles_grid.imprint_building(building, grid_position, building_entity);
    building_entity
}

pub fn get_energy_relay_sprite_bundle(coords: GridCoords) -> SpriteBundle {
    let world_position = coords.to_world_position().extend(0.);
    SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1., 1., 1.),
            custom_size: Some(Vec2::new(ENERGY_RELAY_WORLD_WIDTH, ENERGY_RELAY_WORLD_HEIGHT)),
            ..Default::default()
        },
        transform: Transform::from_translation(world_position + Vec3::new(ENERGY_RELAY_WORLD_WIDTH/2., ENERGY_RELAY_WORLD_HEIGHT/2., 0.0)),
        ..Default::default()
    }
}

pub fn get_energy_relay_grid_imprint() -> GridImprint {
    GridImprint::Rectangle { width: ENERGY_RELAY_GRID_WIDTH, height: ENERGY_RELAY_GRID_HEIGHT }
}