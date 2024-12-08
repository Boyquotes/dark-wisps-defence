use core::panic;
use std::fs::File;
use serde::{Deserialize, Serialize};
use crate::prelude::*;
use crate::buildings::energy_relay::BuilderEnergyRelay;
use crate::buildings::exploration_center::BuilderExplorationCenter;
use crate::buildings::main_base::BuilderMainBase;
use crate::buildings::tower_blaster::BuilderTowerBlaster;
use crate::buildings::tower_cannon::BuilderTowerCannon;
use crate::buildings::tower_rocket_launcher::BuilderTowerRocketLauncher;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::inventory::objectives::{BuilderObjective, ObjectiveDetails};
use crate::map_objects::dark_ore::{BuilderDarkOre, DARK_ORE_GRID_IMPRINT};
use crate::map_objects::quantum_field::BuilderQuantumField;
use crate::map_objects::walls::{BuilderWall, WALL_GRID_IMPRINT};

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
    commands: &mut Commands,
    obstacle_grid: &mut ObstacleGrid,
    almanach: &Almanach,
) {
    map.walls.iter().for_each(|wall_coords| {
        let wall_entity = commands.spawn_empty().id();
        obstacle_grid.imprint(*wall_coords, Field::Wall(wall_entity), WALL_GRID_IMPRINT);
        commands.queue(BuilderWall::new(wall_entity,*wall_coords));
    });
    let _dark_ores = map.dark_ores.iter().map(|dark_ore_coords| {
        let dark_ore_entity = commands.spawn_empty().id();
        obstacle_grid.imprint(*dark_ore_coords, Field::DarkOre(dark_ore_entity), DARK_ORE_GRID_IMPRINT);
        commands.queue(BuilderDarkOre::new(dark_ore_entity, *dark_ore_coords));
        (*dark_ore_coords, dark_ore_entity)
    }).collect::<HashMap<_,_>>();
    map.buildings.iter().for_each(|building| {
        let building_entity = match building.building_type {
            BuildingType::MainBase => {
                let entity = commands.spawn_empty().id();
                commands.queue(BuilderMainBase::new(entity, building.coords));
                entity
            }
            BuildingType::EnergyRelay => {
                let entity = commands.spawn_empty().id();
                commands.queue(BuilderEnergyRelay::new(entity, building.coords));
                entity
            }
            BuildingType::ExplorationCenter => {
                let entity = commands.spawn_empty().id();
                commands.queue(BuilderExplorationCenter::new(entity, building.coords));
                entity
            }
            BuildingType::Tower(tower_type) => {
                match tower_type {
                    TowerType::Blaster => {
                        let entity = commands.spawn_empty().id();
                        commands.queue(BuilderTowerBlaster::new(entity, building.coords));
                        entity
                    },
                    TowerType::Cannon => {
                        let entity = commands.spawn_empty().id();
                        commands.queue(BuilderTowerCannon::new(entity, building.coords));
                        entity
                    },
                    TowerType::RocketLauncher => {
                        let entity = commands.spawn_empty().id();
                        commands.queue(BuilderTowerRocketLauncher::new(entity, building.coords));
                        entity
                    },
                    TowerType::Emitter => {
                        let entity = commands.spawn_empty().id();
                        commands.queue(BuilderTowerBlaster::new(entity, building.coords));
                        entity
                    },
                }
            }
            BuildingType::MiningComplex => {
                // TODO: This won't work as MiningComplex needs special place(type) on obstacle grid, see placing code
                panic!("Not implemented, read the comment");
                // let entity = commands.spawn_empty().id();
                // commands.queue(BuilderMiningComplex::new(entity, building.coords));
                // entity
            }
        };
        obstacle_grid.imprint(building.coords, Field::Building(building_entity, building.building_type, default()), almanach.get_building_info(building.building_type).grid_imprint);
    });
    map.quantum_fields.iter().for_each(|quantum_field| {
        let quantum_field_entity = commands.spawn_empty().id();
        let grid_imprint = GridImprint::Rectangle { width: quantum_field.size, height: quantum_field.size };
        obstacle_grid.imprint(quantum_field.coords, Field::QuantumField(quantum_field_entity), grid_imprint);
        commands.queue(BuilderQuantumField::new(quantum_field_entity, quantum_field.coords, grid_imprint));
    });
    map.objectives.into_iter().for_each(|objective_details| {
       let objective_entity = commands.spawn_empty().id();
       commands.queue(BuilderObjective::new(objective_entity, objective_details));
    });
}