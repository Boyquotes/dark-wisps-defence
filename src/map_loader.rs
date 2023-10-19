use std::fs::File;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::main_base::create_main_base;
use crate::buildings::tower_blaster::create_tower_blaster;
use crate::grids::common::GridCoords;
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
    mut grid: &mut ResMut<ObstacleGrid>,
) {
    map.buildings.iter().for_each(|building| {
        match building.building_type {
            BuildingType::MainBase => {
                create_main_base(&mut commands, &mut grid, building.coords);
            }
            BuildingType::Tower(tower_type) => {
                match tower_type {
                    TowerType::Blaster => {
                        create_tower_blaster(&mut commands, &mut grid, building.coords);
                    }
                }
            }
        }
    });
    map.walls.iter().for_each(|wall_coords| {
        create_wall(&mut commands, &mut grid, *wall_coords);
    });
}