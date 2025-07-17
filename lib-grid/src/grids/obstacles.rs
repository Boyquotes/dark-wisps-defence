use crate::lib_prelude::*;
use crate::grids::base::BaseGrid;

pub struct ObstaclesGridPlugin;
impl Plugin for ObstaclesGridPlugin {
    fn build(&self, app: &mut App) {
        let mut obstacle_grid = ObstacleGrid::new_empty();
        obstacle_grid.resize_and_reset((100, 100));
        app.insert_resource(obstacle_grid);
    }
}

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
    //If there is BelowField, turn into it. If not, become empty
    pub fn clear_main_floor(&mut self) {
        *self = match self {
            Field::Building(_, _, below_field) => (*below_field).into(),
            _ => Field::Empty,
        };
    }
}
impl From<BelowField> for Field {
    fn from(below_field: BelowField) -> Self {
        match below_field {
            BelowField::Empty => Field::Empty,
            BelowField::DarkOre(entity) => Field::DarkOre(entity),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
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
    // Naive imprint that replaced the fields without regards to anything above or under
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
    pub fn deprint_all(&mut self, coords: GridCoords, imprint: GridImprint) {
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
    pub fn deprint_main_floor(&mut self, coords: GridCoords, imprint: GridImprint) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let index = self.index(coords.shifted((x, y)));
                        self.grid[index].clear_main_floor();
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    }
    // Naive reprint that deprints all in old coords and hard imprints in new coords
    pub fn reprint(&mut self, old_coords: GridCoords, new_coords: GridCoords, field: Field, imprint: GridImprint) {
        self.deprint_all(old_coords, imprint);
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
    pub fn imprint_query_count(&self, coords: GridCoords, imprint: GridImprint, query: fn(&Field) -> bool) -> usize {
        let mut count = 0;
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = coords.shifted((x, y));
                        if inner_coords.is_in_bounds(self.bounds()) && query(&self[inner_coords]) {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }
    pub fn imprint_query_element<T>(&self, coords: GridCoords, imprint: GridImprint, query: fn(&Field) -> Option<T>) -> Vec<T> {
        let mut vec = Vec::new();
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = coords.shifted((x, y));
                        if inner_coords.is_in_bounds(self.bounds()) {
                            if let Some(element) = query(&self[inner_coords]) {
                                vec.push(element);
                            }
                        }
                    }
                }
            }
        }
        vec
    }

    pub fn query_building_placement(&self, coords: GridCoords, building_type: BuildingType, imprint: GridImprint) -> bool {
        match building_type {
            BuildingType::MiningComplex => {
                //MiningComplex requires at least one DarkOre cell and no other obstacles
                let GridImprint::Rectangle{ width, height } = imprint;
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

    pub fn imprint_custom(&mut self, coords: GridCoords, imprint: GridImprint, imprint_fn: &dyn Fn(&mut Field)) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let index = self.index(coords.shifted((x, y)));
                        imprint_fn(&mut self.grid[index]);
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    } 
}