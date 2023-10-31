use bevy::prelude::*;
use crate::buildings::common_components::Building;
use crate::grids::base::{BaseGrid, GridVersion};
use crate::grids::common::{GridCoords};
use crate::grids::obstacles::ObstacleGrid;
use crate::search::flooding::{flood_emissions, FloodEmissionsDetails};

#[derive(Component)]
pub struct EmitterEnergy(pub FloodEmissionsDetails);

#[derive(Event)]
pub struct EmitterCreatedEvent {
    pub coords: Vec<GridCoords>,
    pub emissions_details: Vec<FloodEmissionsDetails>,
}

#[derive(Resource)]
pub struct EmissionsEnergyRecalculateAll(pub bool);

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
    pub fn reset_energy_emissions(&mut self) {
        self.grid.iter_mut().for_each(|emissions| {
            emissions.energy = 0.;
        })
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
                        if max_emission == 0. || emissions.energy == 0. {
                            0
                        } else {
                            255 - ((emissions.energy / max_emission).powf(0.1) * 255.) as u8
                        }
                    };
                    chunk[0] = 0;
                    chunk[1] = value;
                    chunk[2] = value;
                }
            }
            idx += 1;
        });
    }
}

pub fn on_emitter_created_system(
    mut events: EventReader<EmitterCreatedEvent>,
    mut emissions_grid: ResMut<EmissionsGrid>,
    obstacle_grid: Res<ObstacleGrid>,
) {
    for event in events.iter() {
        flood_emissions(
            &mut emissions_grid,
            &obstacle_grid,
            &event.coords,
            &event.emissions_details,
            false
        );
    }
}

pub fn emissions_energy_recalculate_all_system(
    mut recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut emissions_grid: ResMut<EmissionsGrid>,
    obstacle_grid: Res<ObstacleGrid>,
    emitters_buildings: Query<(&EmitterEnergy, &Building, &GridCoords)>,
) {
    if !recalculate_all.0 { return; }
    recalculate_all.0 = false;

    emissions_grid.reset_energy_emissions();
    for (emitter, building, coords) in emitters_buildings.iter() {
        flood_emissions(
            &mut emissions_grid,
            &obstacle_grid,
            &building.grid_imprint.covered_coords(*coords),
            &vec![emitter.0.clone()],
            false,
        );
    }
}

