use std::fs::File;
use bevy::prelude::*;
use bevy::utils::HashSet;
use crate::grid::{Field, ObstacleGrid, GridCoords};
use crate::buildings::components::{Building};
use crate::map_loader;
use crate::map_loader::MapBuilding;
use crate::map_objects::walls::{create_wall, remove_wall};
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::{GridObjectPlacer};

pub struct MapEditorPlugin;
impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapInfo::default());
        app.add_systems(Update, editor_onclick_wall_spawn_system);
        app.add_systems(Update, save_map_system);

    }
}

#[derive(Resource, Default)]
pub struct MapInfo {
    pub width: i32,
    pub height: i32,
    pub name: String,
}
impl MapInfo {
    pub fn bounds(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}

pub fn editor_onclick_wall_spawn_system(
    mut commands: Commands,
    mut grid: ResMut<ObstacleGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
) {
    if *grid_object_placer.single() != GridObjectPlacer::Wall { return; }
    let mouse_coords = mouse_info.grid_coords;
    if !mouse_coords.is_in_bounds(grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a wall
        if grid[mouse_coords].is_empty() {
            create_wall(&mut commands, &mut grid, mouse_coords);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a wall
        if grid[mouse_coords].is_wall() {
            remove_wall(&mut commands, &mut grid, mouse_coords);
        }
    }
}

// Save current map to yaml file after 'S' is pressed
pub fn save_map_system(
    map_info: Res<MapInfo>,
    grid: Res<ObstacleGrid>,
    keys: Res<Input<KeyCode>>,
    buildings_query: Query<(&Building, &GridCoords)>,
) {
    if !keys.just_pressed(KeyCode::S) { return; }
    let mut processed_entities = HashSet::new();
    let mut walls = Vec::new();
    let mut buildings = Vec::new();
    // Iterate over grid collecting entities
    for y in 0..grid.height {
        for x in 0..grid.width {
            let coords = GridCoords { x, y };
            match grid[coords] {
                Field::Wall(_) => {
                    walls.push(coords);
                },
                Field::Building(entity) => {
                    if processed_entities.insert(entity) {
                        let (building, building_coords) = buildings_query.get(entity).unwrap();
                        buildings.push(
                            MapBuilding {
                                building_type: building.building_type,
                                coords: *building_coords,
                            }
                        );
                    }
                }
                _ => {}
            }
        }
    }

    let map = map_loader::Map {
        width: map_info.width,
        height: map_info.height,
        buildings,
        walls,
    };
    // Save yaml file
    serde_yaml::to_writer(File::create(format!("maps/{}.yaml", map_info.name)).unwrap(), &map).unwrap();
}