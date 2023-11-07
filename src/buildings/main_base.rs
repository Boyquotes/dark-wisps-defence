use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, MarkerMainBase, TechnicalState};
use crate::common_components::Health;
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::grids::emissions::{EmissionsType, EmitterCreatedEvent, EmitterEnergy};
use crate::grids::energy_supply::{SupplierCreatedEvent, SupplierEnergy};
use crate::grids::obstacles::ObstacleGrid;
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator};

const MAIN_BASE_GRID_WIDTH: i32 = 6;
const MAIN_BASE_GRID_HEIGHT: i32 = 4;
const MAIN_BASE_WORLD_WIDTH: f32 = CELL_SIZE * MAIN_BASE_GRID_WIDTH as f32;
const MAIN_BASE_WORLD_HEIGHT: f32 = CELL_SIZE * MAIN_BASE_GRID_HEIGHT as f32;

pub fn create_main_base(
    commands: &mut Commands,
    emitter_created_event_writer: &mut EventWriter<EmitterCreatedEvent>,
    supplier_created_event_writer: &mut EventWriter<SupplierCreatedEvent>,
    obstacles_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) -> Entity {
    let building = Building {
        grid_imprint: get_main_base_grid_imprint(),
        building_type: BuildingType::MainBase
    };
    let energy_emissions_details = FloodEmissionsDetails {
        emissions_type: EmissionsType::Energy,
        range: usize::MAX,
        evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
    };
    let supplier_energy = SupplierEnergy { range: 15 };
    let building_entity = commands.spawn((
        get_main_base_sprite_bundle(grid_position),
        MarkerMainBase,
        grid_position,
        Health(10000),
        building.clone(),
        EmitterEnergy(energy_emissions_details.clone()),
        supplier_energy,
        TechnicalState{ has_energy_supply: true },
    )).id();
    let covered_coords = building.grid_imprint.covered_coords(grid_position);
    emitter_created_event_writer.send(EmitterCreatedEvent {
        coords: covered_coords.clone(),
        emissions_details: vec![energy_emissions_details],
    });
    supplier_created_event_writer.send(SupplierCreatedEvent {
        coords: covered_coords,
        supplier: supplier_energy,
    });
    obstacles_grid.imprint_building(building, grid_position, building_entity);
    building_entity
}

pub fn get_main_base_sprite_bundle(coords: GridCoords) -> SpriteBundle {
    let world_position = coords.to_world_position().extend(0.);
    SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(MAIN_BASE_WORLD_WIDTH, MAIN_BASE_WORLD_HEIGHT)),
            ..Default::default()
        },
        transform: Transform::from_translation(world_position + Vec3::new(MAIN_BASE_WORLD_WIDTH/2., MAIN_BASE_WORLD_HEIGHT/2., 0.0)),
        ..Default::default()
    }
}

pub fn get_main_base_grid_imprint() -> GridImprint {
    GridImprint::Rectangle { width: MAIN_BASE_GRID_WIDTH, height: MAIN_BASE_GRID_HEIGHT }
}