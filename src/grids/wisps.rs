use bevy::prelude::*;
use crate::grids::base::{BaseGrid, GridVersion};
use crate::grids::common::{GridCoords};
use crate::wisps::components::WispEntity;

pub type WispsGrid = BaseGrid<Vec<WispEntity>, GridVersion>;

impl WispsGrid {
    pub fn wisp_add(&mut self, coords: GridCoords, wisp: WispEntity) {
        self[coords].push(wisp);
        self.version = self.version.wrapping_add(1);
    }
    pub fn wisp_remove(&mut self, coords: GridCoords, wisp: WispEntity) {
        let pos = self[coords].iter().position(|x| *x == wisp).unwrap();
        self[coords].swap_remove(pos);
        self.version = self.version.wrapping_add(1);
    }
    pub fn wisp_move(&mut self, from_coords: GridCoords, to_coords: GridCoords, wisp: WispEntity) {
        self.wisp_remove(from_coords, wisp);
        self.wisp_add(to_coords, wisp);
    }
}