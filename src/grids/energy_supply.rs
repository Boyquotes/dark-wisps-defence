use crate::prelude::*;
use crate::grids::base::{BaseGrid, GridVersion};
use crate::search::flooding::{flood_energy_supply, FloodEnergySupplyMode};

#[derive(Component, Copy, Clone, Debug)]
pub struct SupplierEnergy{
    pub range: usize,
}

#[derive(Event)]
pub struct SupplierChangedEvent {
    pub supplier: Entity,
    pub coords: Vec<GridCoords>,
    pub range: usize,
    pub mode: FloodEnergySupplyMode,
}

#[derive(Clone, Debug, Default)]
pub struct EnergySupplyField(HashSet<Entity>);
impl EnergySupplyField {
    pub fn has_supply(&self) -> bool { self.0.len() > 0 }
    pub fn add_supplier(&mut self, supplier: Entity) { self.0.insert(supplier); }
    pub fn remove_supplier(&mut self, supplier: Entity) { self.0.remove(&supplier); }
    pub fn has_supplier(&self, supplier: Entity) -> bool { self.0.contains(&supplier) }
}

pub type EnergySupplyGrid = BaseGrid<EnergySupplyField, GridVersion>;

impl EnergySupplyGrid {
    pub fn add_supplier(&mut self, coords: GridCoords, supplier: Entity) {
        self[coords].add_supplier(supplier);
        self.version = self.version.wrapping_add(1);
    }
    pub fn remove_supplier(&mut self, coords: GridCoords, supplier: Entity) {
        self[coords].remove_supplier(supplier);
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
}

pub fn on_supplier_changed_system(
    mut events: EventReader<SupplierChangedEvent>,
    mut energy_supply_grid: ResMut<EnergySupplyGrid>,
) {
    for event in events.read() {
        flood_energy_supply(
            &mut energy_supply_grid,
            &event.coords,
            event.mode,
            event.range,
            event.supplier,
        );
    }
}
