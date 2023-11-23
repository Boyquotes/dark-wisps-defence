use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::Building;
use crate::grids::base::{BaseGrid, GridVersion};
use crate::grids::common::{ GridCoords, GridImprint};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Field {
    #[default]
    Empty,
    Building(Entity, BuildingType),
    Wall(Entity),
    DarkOre(Entity),
}
impl Field {
    pub fn is_empty(&self) -> bool {
        matches!(self, Field::Empty)
    }
    pub fn is_building(&self) -> bool {
        matches!(self, Field::Building(_, _))
    }
    pub fn is_wall(&self) -> bool {
        matches!(self, Field::Wall(_))
    }
    pub fn is_dark_ore(&self) -> bool { matches!(self, Field::DarkOre(_)) }
    pub fn is_obstacle(&self) -> bool { !self.is_empty() }
    pub fn is_natural_obstacle(&self) -> bool { !self.is_empty() && !self.is_building() }
}

pub type ObstacleGrid = BaseGrid<Field, GridVersion>;

impl ObstacleGrid {
    pub fn imprint(&mut self, coords: GridCoords, field: Field, imprint: GridImprint) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let index = self.index(coords.shifted((x, y)));
                        self.grid[index] = field.clone();
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    }
    pub fn remove(&mut self, coords: GridCoords, imprint: GridImprint) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let index = self.index(coords.shifted((x, y)));
                        self.grid[index] = Field::Empty;
                    }
                }
            }
        }
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