use bevy::prelude::*;
use crate::grids::base::BaseGrid;
use crate::grids::common::{GridCoords};

pub type WispsGrid = BaseGrid<Vec<Entity>>;

impl WispsGrid {
    pub fn wisp_add(&mut self, coords: GridCoords, wisp: Entity) {
        self[coords].push(wisp);
    }
    pub fn wisp_remove(&mut self, coords: GridCoords, wisp: Entity) {
        let pos = self[coords].iter().position(|x| *x == wisp).unwrap();
        self[coords].swap_remove(pos);
    }
    pub fn wisp_move(&mut self, from_coords: GridCoords, to_coords: GridCoords, wisp: Entity) {
        self.wisp_remove(from_coords, wisp);
        self.wisp_add(to_coords, wisp);
    }
}