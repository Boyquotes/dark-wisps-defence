use std::fs::File;
use bevy::prelude::*;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::energy_relay::BundleEnergyRelay;
use crate::buildings::main_base::{BundleMainBase};
use crate::buildings::mining_complex::{BundleMiningComplex};
use crate::buildings::tower_blaster::{BundleTowerBlaster};
use crate::buildings::tower_cannon::BundleTowerCannon;
use crate::buildings::tower_rocket_launcher::{BundleTowerRocketLauncher};
use crate::grids::common::GridCoords;
use crate::grids::emissions::{EmissionsEnergyRecalculateAll, EmitterChangedEvent};
use crate::grids::energy_supply::{EnergySupplyGrid, SupplierChangedEvent};
use crate::grids::obstacles::ObstacleGrid;
use crate::map_objects::dark_ore::create_dark_ore;
use crate::map_objects::walls::create_wall;

/// Represents yaml content for a map
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub buildings: Vec<MapBuilding>,
    pub walls: Vec<GridCoords>,
    pub dark_ores: Vec<GridCoords>,
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
    asset_server: &AssetServer,
    mut emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    mut emitter_created_event_writer: &mut EventWriter<EmitterChangedEvent>,
    mut supplier_created_event_writer: &mut EventWriter<SupplierChangedEvent>,
    mut obstacles_grid: &mut ResMut<ObstacleGrid>,
    energy_supply_grid: &EnergySupplyGrid,
) {
    map.walls.iter().for_each(|wall_coords| {
        create_wall(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacles_grid, *wall_coords);
    });
    let dark_ores = map.dark_ores.iter().map(|dark_ore_coords| {
        let dark_ore_entity = create_dark_ore(&mut commands, &asset_server, &mut emissions_energy_recalculate_all, &mut obstacles_grid, *dark_ore_coords);
        (*dark_ore_coords, dark_ore_entity)
    }).collect::<HashMap<_,_>>();
    map.buildings.iter().for_each(|building| {
        match building.building_type {
            BuildingType::MainBase => {
                BundleMainBase::new(building.coords, &asset_server)
                    .spawn(&mut commands, &mut emitter_created_event_writer, &mut supplier_created_event_writer, &mut obstacles_grid);
            }
            BuildingType::EnergyRelay => {
                BundleEnergyRelay::new(building.coords)
                    .spawn(&mut commands, &mut emitter_created_event_writer, &mut supplier_created_event_writer, &mut obstacles_grid);
            }
            BuildingType::Tower(tower_type) => {
                match tower_type {
                    TowerType::Blaster => {
                        BundleTowerBlaster::new(building.coords, &asset_server)
                            .update_energy_supply(&energy_supply_grid)
                            .spawn(&mut commands, &mut obstacles_grid);
                    },
                    TowerType::Cannon => {
                        BundleTowerCannon::new(building.coords, asset_server)
                            .update_energy_supply(&energy_supply_grid)
                            .spawn(&mut commands, &mut obstacles_grid);
                    },
                    TowerType::RocketLauncher => {
                        BundleTowerRocketLauncher::new(building.coords, asset_server)
                            .update_energy_supply(&energy_supply_grid)
                            .spawn(&mut commands,&mut obstacles_grid);
                    }
                }
            }
            BuildingType::MiningComplex => {
                BundleMiningComplex::new(building.coords, &asset_server)
                    .update_energy_supply(&energy_supply_grid)
                    .spawn(&mut commands, &mut obstacles_grid, *dark_ores.get(&building.coords).unwrap());
            }
        }
    });
}