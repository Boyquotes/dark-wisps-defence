use std::ops::{Index, IndexMut};
use bevy::prelude::*;
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::ui::UiConfig;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Field {
    #[default]
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
        self.grid = vec![Default::default(); (width * height) as usize];
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
