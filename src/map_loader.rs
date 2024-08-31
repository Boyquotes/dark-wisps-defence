use std::fs::File;
use crate::prelude::*;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::energy_relay::BuilderEnergyRelay;
use crate::buildings::exploration_center::BuilderExplorationCenter;
use crate::buildings::main_base::BuilderMainBase;
use crate::buildings::mining_complex::BuilderMiningComplex;
use crate::buildings::tower_blaster::BuilderTowerBlaster;
use crate::buildings::tower_cannon::BuilderTowerCannon;
use crate::buildings::tower_rocket_launcher::BuilderTowerRocketLauncher;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::emissions::EmissionsEnergyRecalculateAll;
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::inventory::objectives::{BuilderObjective, ObjectiveDetails, ObjectivesCheckInactiveFlag};
use crate::map_objects::dark_ore::BuilderDarkOre;
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
    mut obstacle_grid: &mut ObstacleGrid,
) {
    map.walls.iter().for_each(|wall_coords| {
        BuilderWall::new(*wall_coords).spawn(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid);
    });
    let dark_ores = map.dark_ores.iter().map(|dark_ore_coords| {
        let dark_ore_entity = BuilderDarkOre::new(*dark_ore_coords, &asset_server)
            .spawn(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid);
        (*dark_ore_coords, dark_ore_entity)
    }).collect::<HashMap<_,_>>();
    map.buildings.iter().for_each(|building| {
        let building_entity = match building.building_type {
            BuildingType::MainBase => {
                let mut builder = BuilderMainBase::new(building.coords);
                let entity = builder.entity.get(&mut commands);
                commands.add(builder);
                entity
            }
            BuildingType::EnergyRelay => {
                let mut builder = BuilderEnergyRelay::new(building.coords);
                let entity = builder.entity.get(&mut commands);
                commands.add(builder);
                entity
            }
            BuildingType::ExplorationCenter => {
                let mut builder = BuilderExplorationCenter::new(building.coords);
                let entity = builder.entity.get(&mut commands);
                commands.add(builder);
                entity
            }
            BuildingType::Tower(tower_type) => {
                match tower_type {
                    TowerType::Blaster => {
                        let mut builder = BuilderTowerBlaster::new(building.coords);
                        let entity = builder.entity.get(&mut commands);
                        commands.add(builder);
                        entity
                    },
                    TowerType::Cannon => {
                        let mut builder = BuilderTowerCannon::new(building.coords);
                        let entity = builder.entity.get(&mut commands);
                        commands.add(builder);
                        entity
                    },
                    TowerType::RocketLauncher => {
                        let mut builder = BuilderTowerRocketLauncher::new(building.coords);
                        let entity = builder.entity.get(&mut commands);
                        commands.add(builder);
                        entity
                    }
                }
            }
            BuildingType::MiningComplex => {
                let mut builder = BuilderMiningComplex::new(building.coords);
                let entity = builder.entity.get(&mut commands);
                commands.add(builder);
                entity
                // TODO: This won't work as MiningComplex needs special place(type) on obstacle grid, see placing code
            }
        };
        obstacle_grid.imprint(building.coords, Field::Building(building_entity, building.building_type), building.building_type.grid_imprint());
    });
    map.quantum_fields.iter().for_each(|quantum_field| {
        BuilderQuantumField::new(quantum_field.coords, GridImprint::Rectangle { width: quantum_field.size, height: quantum_field.size })
            .spawn(&mut commands, &mut obstacle_grid);
    });
    map.objectives.into_iter().for_each(|objective_details| {
       BuilderObjective::new(objective_details)
           .spawn(&mut commands, &mut objectives_check_inactive_flag);
    });
}