use crate::lib_prelude::*;
use crate::grids::base::BaseGrid;

pub struct ObstaclesGridPlugin;
impl Plugin for ObstaclesGridPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ObstacleGrid::new_empty())
            .init_resource::<ReservedCoords>()
            .add_systems(First, ReservedCoords::clear_system)
            ;
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Field {
    pub dark_ore: Option<Entity>,
    pub quantum_field: Option<Entity>,
    pub structure: GridStructureType,
}
impl Field {
    pub fn has_dark_ore(&self) -> bool {
        self.dark_ore.is_some()
    }
    pub fn is_within_quantum_field(&self) -> bool {
        self.quantum_field.is_some()
    }
    pub fn is_empty(&self) -> bool {
        matches!(self.structure, GridStructureType::Empty) && !self.is_within_quantum_field() && !self.has_dark_ore()
    }
    pub fn has_building(&self) -> bool {
        matches!(self.structure, GridStructureType::Building(..))
    }
    pub fn has_wall(&self) -> bool {
        matches!(self.structure, GridStructureType::Wall(_))
    }
    pub fn has_structure(&self) -> bool { 
        matches!(self.structure, GridStructureType::Wall(_) | GridStructureType::Building(..)) 
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum GridStructureType {
    #[default]
    Empty,
    Building(Entity, BuildingType),
    Wall(Entity),
}

pub type ObstacleGrid = BaseGrid<Field, GridVersion>;

impl ObstacleGrid {
    pub fn imprint_structure(&mut self, coords: GridCoords, imprint: GridImprint, structure: GridStructureType) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let index = self.index(coords.shifted((x, y)));
                        self.grid[index].structure = structure.clone();
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    }
    pub fn deprint_structure(&mut self, coords: GridCoords, imprint: GridImprint) {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let index = self.index(coords.shifted((x, y)));
                        self.grid[index].structure = GridStructureType::Empty;
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    }
    // Naive reprint that deprints all in old coords and hard imprints in new coords
    pub fn reprint_structure(&mut self, old_coords: GridCoords, new_coords: GridCoords, imprint: GridImprint, new_structure: GridStructureType) {
        self.deprint_structure(old_coords, imprint);
        self.imprint_structure(new_coords, imprint, new_structure);
    }
    pub fn query_imprint_all(&self, coords: GridCoords, imprint: GridImprint, query: fn(&Field) -> bool) -> bool {
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
    pub fn query_imprint_any(&self, coords: GridCoords, imprint: GridImprint, query: fn(&Field) -> bool) -> bool {
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
    pub fn query_imprint_count(&self, coords: GridCoords, imprint: GridImprint, query: fn(&Field) -> bool) -> usize {
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
    pub fn query_imprint_element<T>(&self, coords: GridCoords, imprint: GridImprint, query: fn(&Field) -> Option<T>) -> Vec<T> {
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
                        
                        let field = &self[inner_coords];
                        if field.is_within_quantum_field() || field.has_structure() { return false }
                        if field.has_dark_ore() { has_dark_ore = true; }
                    }
                }
                return has_dark_ore;
            },
            _ => self.query_imprint_all(coords, imprint, |field| field.is_empty()),
        }
    }

    pub fn imprint_custom(&mut self, coords: GridCoords, imprint: GridImprint, imprint_fn: impl Fn(&mut Field)) {
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


// When placing objects on map sometimes we want to reserve space so the state can be changed async later while ensuring that no other object can be placed there in parallel systems.
// This resourvation are cleared in the Firs schedule of the following frame.
#[derive(Resource, Default)]
pub struct ReservedCoords {
    pub for_structures: HashSet<GridCoords>,
}
impl ReservedCoords {
    fn clear_system(mut reserved_coords: ResMut<ReservedCoords>) {
        reserved_coords.for_structures.clear();
    }
}