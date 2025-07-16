use crate::prelude::*;
use crate::lib_prelude::*;
use crate::grids::base::{BaseGrid, GridVersion};
use crate::grids::obstacles::ObstacleGrid;
use crate::search::flooding::{flood_emissions, FloodEmissionsDetails};


pub struct EmissionsPlugin;
impl Plugin for EmissionsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EmissionsGrid::new_with_size(100, 100))
            .init_resource::<EmissionsEnergyRecalculateAll>()
            .add_event::<EmitterChangedEvent>()
            .add_systems(Update, (
                on_emitter_added_system,
                on_emitter_position_changed_system,
            )
            )
            .add_systems(PostUpdate, (
                emissions_calculations_system,
            ));
    }
}

#[derive(Component)]
pub struct EmitterEnergy(pub FloodEmissionsDetails);

#[derive(Event, Debug)]
pub struct EmitterChangedEvent {
    pub emitter_entity: Entity,
    pub coords: Vec<GridCoords>,
    pub emissions_details: Vec<FloodEmissionsDetails>,
}

#[derive(Resource, Default)]
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
        if self[coords].energy < 0. { self[coords].energy = 0.; }
        self.version.energy = self.version.energy.wrapping_add(1);
    }
    pub fn reset_energy_emissions(&mut self) {
        self.grid.iter_mut().for_each(|emissions| {
            emissions.energy = 0.;
        });
        self.version.energy = self.version.energy.wrapping_add(1);
    }
    pub fn imprint_into_heatmap(&self, heatmap: &mut Vec<u8>, emissions_type: EmissionsType) {
        let (min_emission, max_emission) = match emissions_type {
            EmissionsType::Energy => {
                let (mut min, mut max) = (f32::MAX, f32::MIN);
                for emissions in self.grid.iter() {
                    if emissions.energy != 0. {
                        min = min.min(emissions.energy);
                    }
                    max = max.max(emissions.energy);
                }
                (min, max)
            },
        };
        let emissions_range = max_emission - min_emission;
        let mut idx = 0;
        heatmap.chunks_mut(4).for_each(|chunk| {
            let emissions = &self.grid[idx];
            match emissions_type {
                EmissionsType::Energy => {
                    let value = {
                        if emissions_range == 0. || emissions.energy == 0. {
                            0
                        } else {
                            ((emissions.energy - min_emission) / emissions_range * 255.) as u8
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

fn on_emitter_added_system(
    mut events: EventWriter<EmitterChangedEvent>,
    suppliers: Query<(Entity, &GridCoords, &GridImprint, &EmitterEnergy), Added<EmitterEnergy>>,
) {
    for (entity, grid_coords, grid_imprint, emitter) in suppliers.iter() {
        events.write(EmitterChangedEvent {
            emitter_entity: entity,
            coords: grid_imprint.covered_coords(*grid_coords),
            emissions_details: vec![emitter.0.clone()],
        });
    }
}

fn on_emitter_position_changed_system(
    mut recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    suppliers: Query<Ref<GridCoords>, (Changed<GridCoords>, With<EmitterEnergy>)>,
) {
    for grid_coords in suppliers.iter() {
        if !grid_coords.is_added() {
            recalculate_all.0 = true;
            break;
        }
    }
}

fn emissions_calculations_system(
    mut recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut events: EventReader<EmitterChangedEvent>,
    mut emissions_grid: ResMut<EmissionsGrid>,
    obstacle_grid: Res<ObstacleGrid>,
    emitters_buildings: Query<(&EmitterEnergy, &GridImprint, &GridCoords)>,
) {
    if recalculate_all.0 {
        recalculate_all.0 = false;
        emissions_grid.reset_energy_emissions();
        for (emitter, grid_imprint, coords) in emitters_buildings.iter() {
            flood_emissions(
                &mut emissions_grid,
                &obstacle_grid,
                &grid_imprint.covered_coords(*coords),
                &vec![emitter.0.clone()],
                |field| !field.is_wall(),
            );
        }
    } else {
        for event in events.read() {
            flood_emissions(
                &mut emissions_grid,
                &obstacle_grid,
                &event.coords,
                &event.emissions_details,
                |field| !field.is_wall(),
            );
        }
    }
}