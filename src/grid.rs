use std::ops::{Index, IndexMut};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::ui::UiConfig;

pub const CELL_SIZE: f32 = 16.;

pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_grid_system);
        let mut grid = ObstacleGrid::new_empty();
        grid.resize_and_reset(100, 100);
        app.insert_resource(grid);
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GridType {
    Obstacle,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Field {
    Empty,
    Building(Entity),
    Wall(Entity),
}
impl Field {
    pub fn is_empty(&self) -> bool {
        matches!(self, Field::Empty)
    }
    pub fn is_wall(&self) -> bool {
        matches!(self, Field::Wall(_))
    }
}

#[derive(Resource)]
pub struct ObstacleGrid {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<Field>,
    pub version: u32, // Used to determine whether the grid has changed
}

impl ObstacleGrid {
    pub fn new_empty() -> Self {
        Self {
            width: 0,
            height: 0,
            grid: vec![],
            version: 0,
        }
    }
    pub fn resize_and_reset(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.grid = vec![Field::Empty; (width * height) as usize];
    }
    pub fn imprint_building(&mut self, imprint: GridImprint, coords: GridCoords, building_entity: Entity) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let (x, y) = (coords.x + x, coords.y + y);
                        let index = (y * self.width + x) as usize;
                        self.grid[index] = Field::Building(building_entity);
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    }
    pub fn imprint_wall(&mut self, coords: GridCoords, entity: Entity) {
        let index = (coords.y * self.width + coords.x) as usize;
        self.grid[index] = Field::Wall(entity);
        self.version = self.version.wrapping_add(1);
    }
    pub fn remove_wall(&mut self, coords: GridCoords) {
        let index = (coords.y * self.width + coords.x) as usize;
        self.grid[index] = Field::Empty;
        self.version = self.version.wrapping_add(1);
    }
    pub fn is_imprint_placable(&self, coords: GridCoords, imprint: GridImprint) -> bool {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = GridCoords { x: coords.x + x, y: coords.y + y };
                        if !inner_coords.is_in_bounds(self.bounds()) || !self[inner_coords].is_empty() {
                            return false;
                        }

                    }
                }
            }
        }
        true
    }
    pub fn bounds(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}

impl Index<GridCoords> for ObstacleGrid {
    type Output = Field;

    fn index(&self, coords: GridCoords) -> &Self::Output {
        if !coords.is_in_bounds(self.bounds()) {
            panic!("Index out of bounds");
        }
        let index = (coords.y * self.width + coords.x) as usize;
        &self.grid[index]
    }
}
impl IndexMut<GridCoords> for ObstacleGrid {
    fn index_mut(&mut self, coords: GridCoords) -> &mut Self::Output {
        if !coords.is_in_bounds(self.bounds()) {
            panic!("Index out of bounds");
        }
        let index = (coords.y * self.width + coords.x) as usize;
        &mut self.grid[index]
    }
}


pub fn draw_grid_system(grid: Res<ObstacleGrid>, ui_config: Res<UiConfig>, mut gizmos: Gizmos) {
    if !ui_config.show_grid { return; }

    let total_height = grid.height as f32 * CELL_SIZE;
    let total_width = grid.width as f32 * CELL_SIZE;

    // Horizontal lines
    for y in 0..=grid.height {
        let start = Vec2::new(0.0, y as f32 * CELL_SIZE);
        let end = Vec2::new(total_width, y as f32 * CELL_SIZE);
        gizmos.line_2d(start, end, Color::GRAY);
    }

    // Vertical lines
    for x in 0..=grid.width {
        let start = Vec2::new(x as f32 * CELL_SIZE, 0.0);
        let end = Vec2::new(x as f32 * CELL_SIZE, total_height);
        gizmos.line_2d(start, end, Color::GRAY);
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Component)]
pub struct GridCoords {
    pub x: i32,
    pub y: i32,
}

impl GridCoords {
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            x: (transform.translation.x / CELL_SIZE).floor() as i32,
            y: (transform.translation.y / CELL_SIZE).floor() as i32,
        }
    }
    pub fn from_world_vec2(world_coords: Vec2) -> Self {
        Self {
            x: (world_coords.x / CELL_SIZE).floor() as i32,
            y: (world_coords.y / CELL_SIZE).floor() as i32,
        }
    }
    pub fn is_in_bounds(&self, (width, height): (i32, i32)) -> bool {
        self.x >= 0 && self.x < width && self.y >= 0 && self.y < height
    }
    pub fn to_world_coords(&self) -> Vec2 {
        Vec2::new(self.x as f32 * CELL_SIZE, self.y as f32 * CELL_SIZE)
    }
    pub fn to_world_coords_centered(&self) -> Vec2 {
        Vec2::new(self.x as f32 * CELL_SIZE + CELL_SIZE / 2., self.y as f32 * CELL_SIZE + CELL_SIZE / 2.)
    }
}
impl From<(i32, i32)> for GridCoords {
    fn from(coords: (i32, i32)) -> Self {
        Self {
            x: coords.0,
            y: coords.1,
        }
    }
}
impl Into<(i32, i32)> for GridCoords {
    fn into(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridImprint {
    Rectangle{width: i32, height: i32},
}
