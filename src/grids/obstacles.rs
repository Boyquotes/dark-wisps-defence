use bevy::prelude::*;
use crate::grids::base::BaseGrid;
use crate::grids::common::{ GridCoords, GridImprint};

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

pub type ObstacleGrid = BaseGrid<Field>;

impl ObstacleGrid {
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
}