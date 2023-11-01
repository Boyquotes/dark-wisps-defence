use std::fs::File;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::energy_relay::create_energy_relay;
use crate::buildings::main_base::create_main_base;
use crate::buildings::tower_blaster::create_tower_blaster;
use crate::buildings::tower_cannon::create_tower_cannon;
use crate::grids::common::GridCoords;
use crate::grids::emissions::{EmissionsEnergyRecalculateAll, EmissionsGrid, EmitterCreatedEvent};
use crate::grids::obstacles::ObstacleGrid;
use crate::map_objects::walls::create_wall;

/// Represents yaml content for a map
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub buildings: Vec<MapBuilding>,
    pub walls: Vec<GridCoords>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MapBuilding {
    pub building_type: BuildingType,
    pub coords: GridCoords,
}
/// Load a map from a yaml file in the maps directory into a Map struct.
pub fn load_map(map_name: &str) -> Map {
    serde_yaml::from_reader(File::open(format!("maps/{map_name}.yaml")).unwrap()).unwrap()
}

/// Apply the map to the scene.
pub fn apply_map(
    map: Map,
    mut commands: &mut Commands,
    mut emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    mut emitter_created_event_writer: &mut EventWriter<EmitterCreatedEvent>,
    mut obstacles_grid: &mut ResMut<ObstacleGrid>,
) {
    map.walls.iter().for_each(|wall_coords| {
        create_wall(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacles_grid, *wall_coords);
    });
    map.buildings.iter().for_each(|building| {
        match building.building_type {
            BuildingType::MainBase => {
                create_main_base(&mut commands, &mut emitter_created_event_writer, &mut obstacles_grid, building.coords);
            }
            BuildingType::EnergyRelay => {
                create_energy_relay(&mut commands, &mut emitter_created_event_writer, &mut obstacles_grid, building.coords);
            }
            BuildingType::Tower(tower_type) => {
                match tower_type {
                    TowerType::Blaster => {
                        create_tower_blaster(&mut commands, &mut obstacles_grid, building.coords);
                    },
                    TowerType::Cannon => {
                        create_tower_cannon(&mut commands, &mut obstacles_grid, building.coords);
                    }
                }
            }
        }
    });
}