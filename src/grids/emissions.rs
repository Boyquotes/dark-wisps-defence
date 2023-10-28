use bevy::prelude::*;
use crate::grids::base::BaseGrid;
use crate::grids::common::{GridCoords};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EmissionsType {
    Energy,
}

#[derive(Clone, Default, Debug)]
pub struct Emissions {
    pub energy: f32,
}
#[derive(Default)]
pub struct EmissionsGridVersion {
    pub energy: u32,
}

pub type EmissionsGrid = BaseGrid<Emissions, EmissionsGridVersion>;

impl EmissionsGrid {
    pub fn add_energy(&mut self, coords: GridCoords, energy: f32) {
        self[coords].energy += energy;
        self.version.energy = self.version.energy.wrapping_add(1);
    }
}