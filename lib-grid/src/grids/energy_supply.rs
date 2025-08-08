use crate::lib_prelude::*;
use crate::grids::base::BaseGrid;
use crate::search::flooding::{flood_energy_supply, flood_power_coverage, FloodEnergySupplyMode};

pub struct EnergySupplyPlugin;
impl Plugin for EnergySupplyPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EnergySupplyGrid::new_with_size(100, 100))
            .init_resource::<EnergySupplyRecalculatePower>()
            .add_event::<SupplierChangedEvent>()
            .add_systems(PostUpdate, (
                on_supplier_changed_system,
                on_recalculate_power_system.after(on_supplier_changed_system).run_if(resource_changed::<EnergySupplyRecalculatePower>),
            ))
            .add_observer(SupplierEnergy::refresh_on_insert)
            .add_observer(SupplierEnergy::refresh_on_replace)
            .add_observer(GeneratorEnergy::on_insert_for_technical_state);
    }
}

/// Can provide energy to nearby buildings. Does not produce energy.
#[derive(Component, Copy, Clone, Debug)]
#[require(EnergySupplyRange)]
pub struct SupplierEnergy;
impl SupplierEnergy {
    // Detect change in coords or range and trigger supply grid update
    fn refresh_on_insert(
        trigger: Trigger<OnInsert, (GridCoords, EnergySupplyRange)>,
        mut supplier_changed_event_writer: EventWriter<SupplierChangedEvent>,
        suppliers: Query<(&EnergySupplyRange, &GridCoords, &GridImprint), With<SupplierEnergy>>,
    ) {
        let entity = trigger.target();
        let Ok((energy_supply_range, grid_coords, grid_imprint)) = suppliers.get(entity) else { return; };

        supplier_changed_event_writer.write(SupplierChangedEvent {
            supplier: entity,
            coords: grid_imprint.covered_coords(*grid_coords),
            range: *energy_supply_range,
            mode: FloodEnergySupplyMode::Increase,
        });
    }
    // Detect change in coords or range and trigger supply grid update
    fn refresh_on_replace(
        trigger: Trigger<OnReplace, (GridCoords, EnergySupplyRange)>,
        mut supplier_changed_event_writer: EventWriter<SupplierChangedEvent>,
        suppliers: Query<(&EnergySupplyRange, &GridCoords, &GridImprint), With<SupplierEnergy>>,

    ) {
        let entity = trigger.target();
        let Ok((energy_supply_range, grid_coords, grid_imprint)) = suppliers.get(entity) else { return; };

        supplier_changed_event_writer.write(SupplierChangedEvent {
            supplier: entity,
            coords: grid_imprint.covered_coords(*grid_coords),
            range: *energy_supply_range,
            mode: FloodEnergySupplyMode::Decrease,
        });
    }
}

// Produces energy
#[derive(Component, Copy, Clone, Debug)]
pub struct GeneratorEnergy;
impl GeneratorEnergy {
    fn on_insert_for_technical_state(
        trigger: Trigger<OnInsert, GeneratorEnergy>,
        mut technical_states: Query<&mut TechnicalState>,
    ) {
        let Ok(mut technical_state) = technical_states.get_mut(trigger.target()) else { return; };
        technical_state.has_power = true;
    }
}

#[derive(Event)]
pub struct SupplierChangedEvent {
    pub supplier: Entity,
    pub coords: Vec<GridCoords>,
    pub range: EnergySupplyRange,
    pub mode: FloodEnergySupplyMode,
}

#[derive(Resource, Default)]
struct EnergySupplyRecalculatePower(bool);


#[derive(Clone, Debug, Default)]
pub struct EnergySupplyField {
    suppliers: HashSet<Entity>,
    has_power: bool,
}
impl EnergySupplyField {
    // Only checks if there is any supplier, not if it has power
    pub fn has_supply(&self) -> bool { self.suppliers.len() > 0 }
    pub fn add_supplier(&mut self, supplier: Entity) { self.suppliers.insert(supplier); }
    pub fn remove_supplier(&mut self, supplier: Entity) { self.suppliers.remove(&supplier); }
    pub fn has_supplier(&self, supplier: Entity) -> bool { self.suppliers.contains(&supplier) }
    pub fn has_power(&self) -> bool { self.has_power }
    pub fn set_power(&mut self, power: bool) { self.has_power = power; }
    pub fn suppliers(&self) -> &HashSet<Entity> { &self.suppliers }
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
    /// At least one of the imprint's cells must have power.
    pub fn is_imprint_powered(&self, coords: GridCoords, imprint: GridImprint) -> bool {
        match imprint {
            GridImprint::Rectangle { width, height } => {
                for y in 0..height {
                    for x in 0..width {
                        let inner_coords = coords.shifted((x, y));
                        if inner_coords.is_in_bounds(self.bounds()) && self[inner_coords].has_power() {
                            return true;
                        }

                    }
                }
            }
        }
        false
    }
    pub fn reset_all_power_indicators(&mut self) {
        self.grid.iter_mut().for_each(|field| field.set_power(false));
        self.version = self.version.wrapping_add(1);
    }
}

fn on_supplier_changed_system(
    mut need_recalculate_power: ResMut<EnergySupplyRecalculatePower>,
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
        need_recalculate_power.0 = true;
    }
}

fn on_recalculate_power_system(
    mut energy_supply_grid: ResMut<EnergySupplyGrid>,
    mut need_recalculate_power: ResMut<EnergySupplyRecalculatePower>,
    generators_energy: Query<&GridCoords, With<GeneratorEnergy>>,
) {
    if !need_recalculate_power.0 { return; }
    need_recalculate_power.0 = false;

    flood_power_coverage(&mut energy_supply_grid, &generators_energy);
}