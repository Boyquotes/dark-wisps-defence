pub mod obstacles;
pub mod common;
pub mod base;
pub mod wisps;
pub mod visited;
pub mod emissions;
pub mod energy_supply;

use crate::prelude::*;

pub struct GridsPlugin;
impl Plugin for GridsPlugin {
    fn build(&self, app: &mut App) {
        let mut obstacle_grid = obstacles::ObstacleGrid::new_empty();
        obstacle_grid.resize_and_reset(100, 100);
        app.insert_resource(obstacle_grid);
        let mut wisps_grid = wisps::WispsGrid::new_empty();
        wisps_grid.resize_and_reset(100, 100);
        app.insert_resource(wisps_grid);
        let mut emissions_grid = emissions::EmissionsGrid::new_empty();
        emissions_grid.resize_and_reset(100, 100);
        app.insert_resource(emissions_grid);
        app.insert_resource(emissions::EmissionsEnergyRecalculateAll(false));
        app.add_event::<emissions::EmitterChangedEvent>();
        let mut energy_supply_grid = energy_supply::EnergySupplyGrid::new_empty();
        energy_supply_grid.resize_and_reset(100, 100);
        app.insert_resource(energy_supply_grid);
        app.add_event::<energy_supply::SupplierChangedEvent>();

        app.add_systems(PostUpdate, (
            emissions::emissions_calculations_system,
            energy_supply::on_supplier_changed_system,
        ));
    }
}