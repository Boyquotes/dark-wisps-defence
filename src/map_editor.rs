use std::fs::File;

use lib_grid::grids::obstacles::{Field, ObstacleGrid};

use crate::prelude::*;
use crate::objectives::ObjectiveDetails;
use crate::map_loader;
use crate::map_loader::{MapBuilding, MapQuantumField};
use crate::map_objects::dark_ore::DarkOre;
use crate::map_objects::quantum_field::QuantumField;

pub struct MapEditorPlugin;
impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapInfo::default());
        app.add_systems(Update, save_map_system);

    }
}

#[derive(Resource, Default)]
pub struct MapInfo {
    pub grid_width: i32,
    pub grid_height: i32,
    pub world_width: f32,
    pub world_height: f32,
    pub name: String,
}

// Save current map to yaml file after 'S' is pressed
pub fn save_map_system(
    map_info: Res<MapInfo>,
    grid: Res<ObstacleGrid>,
    keys: Res<ButtonInput<KeyCode>>,
    objectives: Query<&ObjectiveDetails>,
    buildings_query: Query<(&BuildingType, &GridCoords), With<Building>>,
    dark_ores_query: Query<(&DarkOre, &GridCoords)>,
    quantum_fields_query: Query<(&GridCoords, &GridImprint), With<QuantumField>>,
) {
    if !keys.just_pressed(KeyCode::KeyS) { return; }
    let mut processed_entities = HashSet::new();
    let mut walls = Vec::new();
    let mut dark_ores = Vec::new();
    let mut buildings = Vec::new();
    let mut quantum_fields = Vec::new();
    // Iterate over grid collecting entities
    for y in 0..grid.height {
        for x in 0..grid.width {
            let coords = GridCoords { x, y };
            match &grid[coords] {
                Field::Wall(_) => {
                    walls.push(coords);
                },
                Field::DarkOre(entity) => {
                    if processed_entities.insert(entity) {
                        let (_, dark_ore_coords) = dark_ores_query.get(*entity).unwrap();
                        dark_ores.push(*dark_ore_coords);
                    }
                },
                Field::Building(entity, _, below_field) => {
                    if !below_field.is_empty() {
                        panic!("BelowFields in editor not yet implemented");
                    }
                    if processed_entities.insert(entity) {
                        let (building_type, building_coords) = buildings_query.get(*entity).unwrap();
                        buildings.push(
                            MapBuilding {
                                building_type: *building_type,
                                coords: *building_coords,
                            }
                        );
                    }
                }
                Field::QuantumField(entity) => {
                    if processed_entities.insert(entity) {
                        let (quantum_field_coords, quantum_field_grid_imprint) = quantum_fields_query.get(*entity).unwrap();
                        let GridImprint::Rectangle { width, .. } = quantum_field_grid_imprint;
                        quantum_fields.push(MapQuantumField { coords: *quantum_field_coords, size: *width });
                    }
                }
                _ => {}
            }
        }
    }

    let map = map_loader::Map {
        width: map_info.grid_width,
        height: map_info.grid_height,
        buildings,
        walls,
        dark_ores,
        quantum_fields,
        objectives: objectives.iter().map(|objective| objective.clone()).collect(),
    };
    // Save yaml file
    serde_yaml::to_writer(File::create(format!("maps/{}.yaml", map_info.name)).unwrap(), &map).unwrap();
}