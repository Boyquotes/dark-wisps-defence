use std::fs::File;
use crate::prelude::*;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::energy_relay::BuilderEnergyRelay;
use crate::buildings::exploration_center::BuilderExplorationCenter;
use crate::buildings::main_base::{BuilderMainBase};
use crate::buildings::mining_complex::{BuilderMiningComplex};
use crate::buildings::tower_blaster::{BuilderTowerBlaster};
use crate::buildings::tower_cannon::BuilderTowerCannon;
use crate::buildings::tower_rocket_launcher::{BuilderTowerRocketLauncher};
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::emissions::{EmissionsEnergyRecalculateAll, EmitterChangedEvent};
use crate::grids::energy_supply::{EnergySupplyGrid, SupplierChangedEvent};
use crate::grids::obstacles::ObstacleGrid;
use crate::inventory::objectives::{BuilderObjective, ObjectiveDetails, ObjectivesCheckInactiveFlag};
use crate::map_objects::dark_ore::{BuilderDarkOre};
use crate::map_objects::quantum_field::BuilderQuantumField;
use crate::map_objects::walls::BuilderWall;

/// Represents yaml content for a map
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub buildings: Vec<MapBuilding>,
    pub walls: Vec<GridCoords>,
    pub dark_ores: Vec<GridCoords>,
    pub quantum_fields: Vec<MapQuantumField>,
    pub objectives: Vec<ObjectiveDetails>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MapBuilding {
    pub building_type: BuildingType,
    pub coords: GridCoords,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MapQuantumField {
    pub coords: GridCoords,
    pub size: i32,
}



/// Load a map from a yaml file in the maps directory into a Map struct.
pub fn load_map(map_name: &str) -> Map {
    serde_yaml::from_reader(File::open(format!("maps/{map_name}.yaml")).unwrap()).unwrap()
}

/// Apply the map to the scene.
pub fn apply_map(
    map: Map,
    mut commands: &mut Commands,
    asset_server: &AssetServer,
    mut objectives_check_inactive_flag: &mut ObjectivesCheckInactiveFlag,
    mut emissions_energy_recalculate_all: &mut EmissionsEnergyRecalculateAll,
    mut emitter_created_event_writer: &mut EventWriter<EmitterChangedEvent>,
    mut supplier_created_event_writer: &mut EventWriter<SupplierChangedEvent>,
    mut obstacles_grid: &mut ObstacleGrid,
    energy_supply_grid: &EnergySupplyGrid,
) {
    map.walls.iter().for_each(|wall_coords| {
        BuilderWall::new(*wall_coords).spawn(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacles_grid);
    });
    let dark_ores = map.dark_ores.iter().map(|dark_ore_coords| {
        let dark_ore_entity = BuilderDarkOre::new(*dark_ore_coords, &asset_server)
            .spawn(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacles_grid);
        (*dark_ore_coords, dark_ore_entity)
    }).collect::<HashMap<_,_>>();
    map.buildings.iter().for_each(|building| {
        match building.building_type {
            BuildingType::MainBase => {
                BuilderMainBase::new(building.coords, &asset_server)
                    .spawn(&mut commands, &mut emitter_created_event_writer, &mut supplier_created_event_writer, &mut obstacles_grid);
            }
            BuildingType::EnergyRelay => {
                BuilderEnergyRelay::new(building.coords, &asset_server)
                    .spawn(&mut commands, &mut emitter_created_event_writer, &mut supplier_created_event_writer, &mut obstacles_grid);
            }
            BuildingType::ExplorationCenter => {
                BuilderExplorationCenter::new(building.coords, &asset_server)
                    .update_energy_supply(&energy_supply_grid)
                    .spawn(&mut commands, &mut obstacles_grid);
            }
            BuildingType::Tower(tower_type) => {
                match tower_type {
                    TowerType::Blaster => {
                        BuilderTowerBlaster::new(building.coords, &asset_server)
                            .update_energy_supply(&energy_supply_grid)
                            .spawn(&mut commands, &mut obstacles_grid);
                    },
                    TowerType::Cannon => {
                        BuilderTowerCannon::new(building.coords, asset_server)
                            .update_energy_supply(&energy_supply_grid)
                            .spawn(&mut commands, &mut obstacles_grid);
                    },
                    TowerType::RocketLauncher => {
                        BuilderTowerRocketLauncher::new(building.coords, asset_server)
                            .update_energy_supply(&energy_supply_grid)
                            .spawn(&mut commands,&mut obstacles_grid);
                    }
                }
            }
            BuildingType::MiningComplex => {
                BuilderMiningComplex::new(building.coords, &asset_server)
                    .update_energy_supply(&energy_supply_grid)
                    .spawn(&mut commands, &mut obstacles_grid, *dark_ores.get(&building.coords).unwrap());
            }
        }
    });
    map.quantum_fields.iter().for_each(|quantum_field| {
        BuilderQuantumField::new(quantum_field.coords, GridImprint::Rectangle { width: quantum_field.size, height: quantum_field.size })
            .spawn(&mut commands, &mut obstacles_grid);
    });
    map.objectives.into_iter().for_each(|objective_details| {
       BuilderObjective::new(objective_details)
           .spawn(&mut commands, &mut objectives_check_inactive_flag);
    });
}