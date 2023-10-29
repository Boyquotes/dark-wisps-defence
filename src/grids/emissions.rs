use bevy::prelude::*;
use crate::grids::base::{BaseGrid, GridVersion};
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
    pub energy: GridVersion,
}

pub type EmissionsGrid = BaseGrid<Emissions, EmissionsGridVersion>;

impl EmissionsGrid {
    pub fn add_energy(&mut self, coords: GridCoords, energy: f32) {
        self[coords].energy += energy;
        self.version.energy = self.version.energy.wrapping_add(1);
    }
    pub fn imprint_into_heatmap(&self, heatmap: &mut Vec<u8>, emissions_type: EmissionsType) {
        let max_emission = match emissions_type {
            EmissionsType::Energy => self.grid.iter().map(|emissions| emissions.energy).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        };
        let mut idx = 0;
        heatmap.chunks_mut(4).for_each(|chunk| {
            let emissions = &self.grid[idx];
            match emissions_type {
                EmissionsType::Energy => {
                    let value = {
                        if max_emission == 0. {
                            0
                        } else if emissions.energy == 0. {
                            255 // The lower the value the higher the emission
                        } else { 255 - (emissions.energy / max_emission * 255.) as u8 }
                    };
                    chunk[0] = value;
                    chunk[1] = value;
                    chunk[2] = value;
                }
            }
            idx += 1;
        });
    }
}