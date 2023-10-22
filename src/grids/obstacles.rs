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
    pub fn is_obstacle(&self) -> bool {
        match self {
            Field::Wall(_) => true,
            Field::Building(_) => true,
            _ => false,
        }
    }
}

pub type ObstacleGrid = BaseGrid<Field>;

impl ObstacleGrid {
    pub fn imprint_building(&mut self, imprint: GridImprint, coords: GridCoords, building_entity: Entity) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let index = self.index(coords.shifted((x, y)));
                        self.grid[index] = Field::Building(building_entity);
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    }
    pub fn imprint_wall(&mut self, coords: GridCoords, entity: Entity) {
        let index = self.index(coords);
        self.grid[index] = Field::Wall(entity);
        self.version = self.version.wrapping_add(1);
    }
    pub fn remove_wall(&mut self, coords: GridCoords) {
        let index = self.index(coords);
        self.grid[index] = Field::Empty;
        self.version = self.version.wrapping_add(1);
    }
    pub fn is_imprint_placable(&self, coords: GridCoords, imprint: GridImprint) -> bool {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = coords.shifted((x, y));
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