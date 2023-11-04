pub mod emissions;
pub mod energy_supply;

use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

pub struct OverlaysPlugin;
impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(emissions::EmissionsOverlayMode::Energy(0));
        app.insert_resource(energy_supply::EnergySupplyOverlayMode::None);
        app.add_plugins((
            Material2dPlugin::<emissions::EmissionHeatmapMaterial>::default(),
            Material2dPlugin::<energy_supply::EnergySupplyHeatmapMaterial>::default(),
        ));
        app.add_systems(Startup, (
            emissions::create_emissions_overlay_startup_system,
            energy_supply::create_energy_supply_overlay_startup_system,
        ));
        app.add_systems(PreUpdate, (
            emissions::update_emissions_overlay_system,
            energy_supply::update_energy_supply_overlay_system,
        ));
        app.add_systems(Update, (
            emissions::manage_emissions_overlay_mode_system,
            energy_supply::manage_energy_supply_overlay_mode_system,
        ));
    }
}
