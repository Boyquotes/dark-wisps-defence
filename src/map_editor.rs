use std::fs::File;

use lib_grid::grids::obstacles::{Field, ObstacleGrid};

use crate::prelude::*;
use crate::map_loader;
use crate::map_loader::{MapBuilding, MapQuantumField};
use crate::map_objects::dark_ore::DarkOre;
use crate::map_objects::quantum_field::QuantumField;

fn register_loaders(mut registry: ResMut<lib_core::common::GameLoadRegistry>) {
    registry.register::<MapInfoLoader>();
}

pub struct MapEditorPlugin;
impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapInfo::default());
        app.add_systems(Update, save_map_system);
        app.add_systems(Startup, register_loaders);
    }
}

use lib_core::common::{Loadable, LoadResult, SSS, Saveable};
use lib_core::common::rusqlite::{self, Transaction};

#[derive(Resource, Default, Clone)]
pub struct MapInfo {
    pub grid_width: i32,
    pub grid_height: i32,
    pub world_width: f32,
    pub world_height: f32,
    pub name: String,
    pub map_file: map_loader::MapFile,
}
impl SSS for MapInfo {}
impl Saveable for MapInfo {
    fn save(self, tx: &Transaction) -> rusqlite::Result<()> {
        tx.execute(
            "INSERT OR REPLACE INTO map_info (id, width, height, name) VALUES (1, ?1, ?2, ?3)",
            (self.grid_width, self.grid_height, &self.name),
        )?;
        Ok(())
    }
}

#[derive(Component)]
pub struct MapInfoLoader;
impl SSS for MapInfoLoader {}
impl Loadable for MapInfoLoader {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult> {
        // Singleton load, only run once
        let mut stmt = ctx.conn.prepare("SELECT width, height, name FROM map_info WHERE id = 1")?;
        let result = stmt.query_row([], |row| {
            let width: i32 = row.get(0)?;
            let height: i32 = row.get(1)?;
            let name: String = row.get(2)?;
            Ok((width, height, name))
        });

        match result {
            Ok((width, height, name)) => {
                let map_info = MapInfo {
                    grid_width: width,
                    grid_height: height,
                    world_width: width as f32 * CELL_SIZE,
                    world_height: height as f32 * CELL_SIZE,
                    name,
                    map_file: map_loader::MapFile::default(),
                };
                ctx.commands.insert_resource(map_info);
                // Singleton loaded, we are done.
                Ok(LoadResult::Finished)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(LoadResult::Finished),
            Err(e) => Err(e),
        }
    }
}

impl MapInfo {
    pub fn set(&mut self, map_file: map_loader::MapFile) {
        self.grid_width = map_file.width;
        self.grid_height = map_file.height;
        self.world_width = map_file.width as f32 * CELL_SIZE;
        self.world_height = map_file.height as f32 * CELL_SIZE;
        self.map_file = map_file;
    }
    pub fn bounds(&self) -> (i32, i32) {
        (self.grid_width, self.grid_height)
    }
}

// Save current map to yaml file after 'S' is pressed
pub fn save_map_system(
    map_info: Res<MapInfo>,
    grid: Res<ObstacleGrid>,
    keys: Res<ButtonInput<KeyCode>>,
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

    let map = map_loader::MapFile {
        width: map_info.grid_width,
        height: map_info.grid_height,
        buildings,
        walls,
        dark_ores,
        quantum_fields,
        objectives: vec![],
        summonings: vec![],
    };
    // Save yaml file
    serde_yaml::to_writer(File::create(format!("maps/{}.yaml", map_info.name)).unwrap(), &map).unwrap();
}