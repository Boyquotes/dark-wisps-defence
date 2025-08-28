use std::fs::File;

use lib_grid::grids::obstacles::{Field, ObstacleGrid};

use crate::map_editor::MapInfo;
use crate::prelude::*;
use crate::buildings::energy_relay::BuilderEnergyRelay;
use crate::buildings::exploration_center::BuilderExplorationCenter;
use crate::buildings::main_base::BuilderMainBase;
use crate::buildings::tower_blaster::BuilderTowerBlaster;
use crate::buildings::tower_cannon::BuilderTowerCannon;
use crate::buildings::tower_emitter::BuilderTowerEmitter;
use crate::buildings::tower_rocket_launcher::BuilderTowerRocketLauncher;
use crate::objectives::ObjectiveDetails;
use crate::map_objects::dark_ore::{BuilderDarkOre, DARK_ORE_GRID_IMPRINT};
use crate::map_objects::quantum_field::BuilderQuantumField;
use crate::map_objects::walls::{BuilderWall, WALL_GRID_IMPRINT};

pub struct MapLoaderPlugin;
impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(LoadMapRequest::on_trigger);
    }
}

#[derive(Event)]
pub struct LoadMapRequest(pub String);
impl LoadMapRequest {
    fn on_trigger(
        trigger: Trigger<LoadMapRequest>,
        mut commands: Commands,
        mut obstacles_grid: ResMut<ObstacleGrid>,
        mut map_info: ResMut<MapInfo>,
        almanach: Res<Almanach>,
    ) {
        let map_name = &trigger.event().0;
        let mut map = Map::load(map_name);
        map_info.name = map_name.clone();
        map_info.grid_width = map.width;
        map_info.grid_height = map.height;
        map_info.world_width = map.width as f32 * CELL_SIZE;
        map_info.world_height = map.height as f32 * CELL_SIZE;
        map.apply(&mut commands, &mut obstacles_grid, &almanach);
    }
}

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
impl Map {
    /// Load a map from a yaml file in the maps directory into a Map struct.
    fn load(map_name: &str) -> Self {
        serde_yaml::from_reader(File::open(format!("maps/{}.yaml", map_name)).unwrap()).unwrap()
    }
    /// Apply the map to the scene.
    fn apply(
        &mut self,
        commands: &mut Commands,
        obstacle_grid: &mut ObstacleGrid,
        almanach: &Almanach,
    ) {
        self.walls.iter().for_each(|wall_coords| {
            let wall_entity = commands.spawn(BuilderWall::new(*wall_coords)).id();
            obstacle_grid.imprint(*wall_coords, Field::Wall(wall_entity), WALL_GRID_IMPRINT);
        });
        let _dark_ores = self.dark_ores.iter().map(|dark_ore_coords| {
            let dark_ore_entity = commands.spawn(BuilderDarkOre::new(*dark_ore_coords)).id();
            obstacle_grid.imprint(*dark_ore_coords, Field::DarkOre(dark_ore_entity), DARK_ORE_GRID_IMPRINT);
            (*dark_ore_coords, dark_ore_entity)
        }).collect::<HashMap<_,_>>();
        self.buildings.iter().for_each(|building| {
            let building_entity = match building.building_type {
                BuildingType::MainBase => commands.spawn(BuilderMainBase::new(building.coords)).id(),
                BuildingType::EnergyRelay => commands.spawn(BuilderEnergyRelay::new(building.coords)).id(),
                BuildingType::ExplorationCenter => commands.spawn(BuilderExplorationCenter::new(building.coords)).id(),
                BuildingType::Tower(tower_type) => {
                    match tower_type {
                        TowerType::Blaster => commands.spawn(BuilderTowerBlaster::new(building.coords)).id(),
                        TowerType::Cannon => commands.spawn(BuilderTowerCannon::new(building.coords)).id(),
                        TowerType::RocketLauncher => commands.spawn(BuilderTowerRocketLauncher::new(building.coords)).id(),
                        TowerType::Emitter => commands.spawn(BuilderTowerEmitter::new(building.coords)).id(),
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
        self.quantum_fields.iter().for_each(|quantum_field| {
            let grid_imprint = GridImprint::Rectangle { width: quantum_field.size, height: quantum_field.size };
            let quantum_field_entity = commands.spawn(BuilderQuantumField::new(quantum_field.coords, grid_imprint)).id();
            obstacle_grid.imprint(quantum_field.coords, Field::QuantumField(quantum_field_entity), grid_imprint);
        });
        self.objectives.drain(..).for_each(|objective_details| {
            commands.spawn(objective_details);
        });
    }
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