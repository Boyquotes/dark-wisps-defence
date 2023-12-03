use bevy::prelude::*;
use crate::grids::base::{BaseGrid, GridVersion};
use crate::grids::common::{GridCoords, GridImprint};
use crate::search::flooding::{flood_energy_supply, FloodEnergySupplyMode};

#[derive(Component, Copy, Clone, Debug)]
pub struct SupplierEnergy{
    pub range: usize,
}

#[derive(Event)]
pub struct SupplierChangedEvent {
    pub coords: Vec<GridCoords>,
    pub supplier: SupplierEnergy,
    pub mode: FloodEnergySupplyMode,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct EnergySupplyField(u32);
impl EnergySupplyField {
    pub fn has_supply(&self) -> bool { self.0 > 0 }
    pub fn increase_supply(&mut self) { self.0 += 1 }
    pub fn decrease_supply(&mut self) { self.0 -= 1 }
}

pub type EnergySupplyGrid = BaseGrid<EnergySupplyField, GridVersion>;

impl EnergySupplyGrid {
    pub fn increase_supply(&mut self, coords: GridCoords) {
        self[coords].increase_supply();
        self.version = self.version.wrapping_add(1);
    }
    pub fn decrease_supply(&mut self, coords: GridCoords) {
        self[coords].decrease_supply();
        self.version = self.version.wrapping_add(1);
    }
    /// At least one of the imprint's cells mut have energy supply.
    pub fn is_imprint_suppliable(&self, coords: GridCoords, imprint: GridImprint) -> bool {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = coords.shifted((x, y));
                        if inner_coords.is_in_bounds(self.bounds()) && self[inner_coords].has_supply() {
                            return true;
                        }

                    }
                }
            }
        }
        false
    }
    pub fn imprint_into_heatmap(&self, heatmap: &mut Vec<u8>) {
        let mut idx = 0;
        heatmap.chunks_mut(4).for_each(|chunk| {
            let has_energy_supply = self.grid[idx].has_supply();
            if has_energy_supply {
                chunk[3] = 15;
            } else {
                chunk[3] = 0;
            }
            idx += 1;
        });
    }
}

pub fn on_supplier_created_system(
    mut events: EventReader<SupplierChangedEvent>,
    mut energy_supply_grid: ResMut<EnergySupplyGrid>,
) {
    for event in events.read() {
        flood_energy_supply(
            &mut energy_supply_grid,
            &event.coords,
            event.mode,
            event.supplier.range
        );
    }
}
