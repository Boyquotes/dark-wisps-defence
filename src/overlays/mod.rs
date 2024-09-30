pub mod emissions;
pub mod energy_supply;

use crate::prelude::*;
use bevy::sprite::Material2dPlugin;

pub struct OverlaysPlugin;
impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(emissions::EmissionsOverlayMode::Energy(0));
        app.add_plugins((
            energy_supply::EnergySupplyOverlayPlugin,
            Material2dPlugin::<emissions::EmissionHeatmapMaterial>::default(),
            Material2dPlugin::<energy_supply::EnergySupplyHeatmapMaterial>::default(),
        ));
        app.add_systems(Startup, (
            emissions::create_emissions_overlay_startup_system,
        ));
        app.add_systems(PreUpdate, (
            emissions::update_emissions_overlay_system,
        ));
        app.add_systems(Update, (
            emissions::manage_emissions_overlay_mode_system,
        ));
    }
}
