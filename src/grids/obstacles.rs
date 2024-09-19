use crate::prelude::*;
use crate::grids::base::{BaseGrid, GridVersion};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Field {
    #[default]
    Empty,
    Building(Entity, BuildingType, BelowField),
    Wall(Entity),
    DarkOre(Entity),
    QuantumField(Entity),
}
impl Field {
    pub fn is_empty(&self) -> bool {
        matches!(self, Field::Empty)
    }
    pub fn is_building(&self) -> bool {
        matches!(self, Field::Building(..))
    }
    pub fn is_wall(&self) -> bool {
        matches!(self, Field::Wall(_))
    }
    pub fn is_dark_ore(&self) -> bool { matches!(self, Field::DarkOre(_)) }
    pub fn is_high_obstacle(&self) -> bool { 
        matches!(self, Field::Wall(_) | Field::Building(..)) 
    }
    pub fn is_natural_obstacle(&self) -> bool {
        matches!(self, Field::Wall(_) | Field::DarkOre(_) | Field::Building(_, BuildingType::MiningComplex, BelowField::DarkOre(_)) | Field::QuantumField(_))
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum BelowField {
    #[default]
    Empty,
    DarkOre(Entity),
}
impl BelowField {
    pub fn is_empty(&self) -> bool {
        matches!(self, BelowField::Empty)
    }
}

pub type ObstacleGrid = BaseGrid<Field, GridVersion>;

impl ObstacleGrid {
    pub fn imprint(&mut self, coords: GridCoords, field: Field, imprint: GridImprint) {
        if matches!(field, Field::Building(_, BuildingType::MiningComplex, _)) { panic!("Don't use `imprint()` with MiningComplex, use `imprint_mining_complex()`"); }
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
    pub fn deprint(&mut self, coords: GridCoords, imprint: GridImprint) {
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
    pub fn reprint(&mut self, old_coords: GridCoords, new_coords: GridCoords, field: Field, imprint: GridImprint) {
        self.deprint(old_coords, imprint);
        self.imprint(new_coords, field, imprint);
    }
    pub fn imprint_query_all(&self, coords: GridCoords, imprint: GridImprint, query: fn(&Field) -> bool) -> bool {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = coords.shifted((x, y));
                        if inner_coords.is_in_bounds(self.bounds()) && !query(&self[inner_coords]) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
    pub fn imprint_query_any(&self, coords: GridCoords, imprint: GridImprint, query: fn(&Field) -> bool) -> bool {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = coords.shifted((x, y));
                        if inner_coords.is_in_bounds(self.bounds()) && query(&self[inner_coords]) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn query_building_placement(&self, coords: GridCoords, building_type: BuildingType, imprint: GridImprint) -> bool {
        match building_type {
            BuildingType::MiningComplex => {
                //MiningComplex requires at least one DarkOre cell and no other obstacles
                let GridImprint::Rectangle{ width, height } = imprint else { panic!("MiningComplex imprint is not a rectangle"); };
                let mut has_dark_ore = false;
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = coords.shifted((x, y));
                        if !inner_coords.is_in_bounds(self.bounds()) { return false; }
                        match &self[inner_coords] {
                            Field::Empty => continue,
                            Field::DarkOre(_) => has_dark_ore = true,
                            _ => return false,
                        }
                    }
                }
                return has_dark_ore;
            },
            _ => self.imprint_query_all(coords, imprint, |field| field.is_empty()),
        }
    }

    // Special imprint version that ensure DarkOre info under the MiningComplex is retained
    pub fn imprint_mining_complex(&mut self, coords: GridCoords, mining_complex_entity: Entity, imprint: GridImprint) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let index = self.index(coords.shifted((x, y)));
                        let below_field = match &self.grid[index] {
                            Field::Empty => BelowField::Empty,
                            Field::DarkOre(entity) => BelowField::DarkOre(*entity),
                            _ => panic!("imprint_mining_complex() can only be used with an Empty or DarkOre Field"),
                        };
                        self.grid[index] = Field::Building(mining_complex_entity, BuildingType::MiningComplex, below_field);
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    }

}