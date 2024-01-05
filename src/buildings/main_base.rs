use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, TechnicalState};
use crate::common::Z_BUILDING;
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::emissions::{EmissionsType, EmitterChangedEvent, EmitterEnergy};
use crate::grids::energy_supply::{SupplierChangedEvent, SupplierEnergy};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode, FloodEnergySupplyMode};

pub const MAIN_BASE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 6, height: 6 };
pub const MAIN_BASE_BASE_IMAGE: &str = "buildings/main_base.png";

#[derive(Component)]
pub struct MarkerMainBase;


#[derive(Bundle)]
pub struct BundleMainBase {
    pub sprite_bundle: SpriteBundle,
    pub marker_main_base: MarkerMainBase,
    pub grid_position: GridCoords,
    pub health: Health,
    pub building: Building,
    pub emitter_energy: EmitterEnergy,
    pub supplier_energy: SupplierEnergy,
    pub technical_state: TechnicalState,
}
impl BundleMainBase {
    pub fn new(grid_position: GridCoords, asset_server: &AssetServer) -> Self {
        Self {
            sprite_bundle: get_main_base_sprite_bundle(grid_position, asset_server),
            marker_main_base: MarkerMainBase,
            grid_position,
            health: Health(10000),
            building: Building::from(BuildingType::MainBase),
            emitter_energy: EmitterEnergy(FloodEmissionsDetails {
                emissions_type: EmissionsType::Energy,
                range: usize::MAX,
                evaluator: FloodEmissionsEvaluator::ExponentialDecay { start_value: 100., decay: 0.1 },
                mode: FloodEmissionsMode::Increase,
            }),
            supplier_energy: SupplierEnergy { range: 15 },
            technical_state: TechnicalState { has_energy_supply: true },
        }
    }
    pub fn spawn(
        self,
        commands: &mut Commands,
        emitter_created_event_writer: &mut EventWriter<EmitterChangedEvent>,
        supplier_created_event_writer: &mut EventWriter<SupplierChangedEvent>,
        obstacles_grid: &mut ResMut<ObstacleGrid>,
    ) -> Entity {
        let grid_position = self.grid_position;
        let covered_coords = MAIN_BASE_GRID_IMPRINT.covered_coords(grid_position);
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

        obstacles_grid.imprint(grid_position, Field::Building(entity, BuildingType::MainBase), MAIN_BASE_GRID_IMPRINT);
        entity
    }
}

pub fn get_main_base_sprite_bundle(coords: GridCoords, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(MAIN_BASE_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(MAIN_BASE_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(MAIN_BASE_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}

pub fn move_main_base(
    emitter_created_event_writer: &mut EventWriter<EmitterChangedEvent>,
    supplier_created_event_writer: &mut EventWriter<SupplierChangedEvent>,
    obstacle_grid: &mut ObstacleGrid,
    main_base: (Entity, &mut GridCoords, &SupplierEnergy, &mut Transform),
    grid_position: GridCoords,
) {
    let (main_base_entity, main_base_location, supplier_energy, transform) = main_base;
    obstacle_grid.reprint(*main_base_location, grid_position, Field::Building(main_base_entity, BuildingType::MainBase), MAIN_BASE_GRID_IMPRINT);
    supplier_created_event_writer.send(SupplierChangedEvent {
        coords: MAIN_BASE_GRID_IMPRINT.covered_coords(*main_base_location),
        supplier: supplier_energy.clone(),
        mode: FloodEnergySupplyMode::Decrease,
    });
    supplier_created_event_writer.send(SupplierChangedEvent {
        coords: MAIN_BASE_GRID_IMPRINT.covered_coords(grid_position),
        supplier: supplier_energy.clone(),
        mode: FloodEnergySupplyMode::Increase,
    });
    emitter_created_event_writer.send(EmitterChangedEvent {
        coords: MAIN_BASE_GRID_IMPRINT.covered_coords(*main_base_location),
        emissions_details: vec![FloodEmissionsDetails {
            emissions_type: EmissionsType::Energy,
            range: usize::MAX,
            evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
            mode: FloodEmissionsMode::Decrease,
        }]
    });
    emitter_created_event_writer.send(EmitterChangedEvent {
        coords: MAIN_BASE_GRID_IMPRINT.covered_coords(grid_position),
        emissions_details: vec![FloodEmissionsDetails {
            emissions_type: EmissionsType::Energy,
            range: usize::MAX,
            evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
            mode: FloodEmissionsMode::Increase,
        }]
    });
    *main_base_location = grid_position;
    transform.translation = grid_position.to_world_position_centered(MAIN_BASE_GRID_IMPRINT).extend(Z_BUILDING);
}
