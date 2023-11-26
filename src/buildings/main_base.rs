use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, MarkerMainBase, TechnicalState};
use crate::common::Z_BUILDING;
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::emissions::{EmissionsType, EmitterCreatedEvent, EmitterEnergy};
use crate::grids::energy_supply::{SupplierCreatedEvent, SupplierEnergy};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator};

pub const MAIN_BASE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 6, height: 6 };

pub fn create_main_base(
    commands: &mut Commands,
    asset_server: &AssetServer,
    emitter_created_event_writer: &mut EventWriter<EmitterCreatedEvent>,
    supplier_created_event_writer: &mut EventWriter<SupplierCreatedEvent>,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) -> Entity {
    let building = Building {
        grid_imprint: MAIN_BASE_GRID_IMPRINT,
        building_type: BuildingType::MainBase
    };
    let energy_emissions_details = FloodEmissionsDetails {
        emissions_type: EmissionsType::Energy,
        range: usize::MAX,
        evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
    };
    let supplier_energy = SupplierEnergy { range: 15 };
    let building_entity = commands.spawn((
        get_main_base_sprite_bundle(grid_position, asset_server),
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
    obstacle_grid.imprint(grid_position, Field::Building(building_entity, building.building_type), MAIN_BASE_GRID_IMPRINT);
    building_entity
}

pub fn get_main_base_sprite_bundle(coords: GridCoords, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(MAIN_BASE_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load("buildings/main_base.png"),
        transform: Transform::from_translation(coords.to_world_position_centered(MAIN_BASE_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}