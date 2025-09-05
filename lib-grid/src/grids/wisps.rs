use crate::lib_prelude::*;
use crate::grids::base::BaseGrid;

pub struct WispsGridPlugin;
impl Plugin for WispsGridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WispsGrid::new_empty());
    }
}

pub type WispsGrid = BaseGrid<Vec<Entity>, GridVersion>;

impl WispsGrid {
    pub fn wisp_add(&mut self, coords: GridCoords, wisp: Entity) {
        self[coords].push(wisp);
        self.version = self.version.wrapping_add(1);
    }
    pub fn wisp_remove(&mut self, coords: GridCoords, wisp: Entity) {
        let pos = self[coords].iter().position(|x| *x == wisp).unwrap();
        self[coords].swap_remove(pos);
        self.version = self.version.wrapping_add(1);
    }
    pub fn wisp_move(&mut self, from_coords: GridCoords, to_coords: GridCoords, wisp: Entity) {
        self.wisp_remove(from_coords, wisp);
        self.wisp_add(to_coords, wisp);
    }
}