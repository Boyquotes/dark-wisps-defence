pub mod emissions;
pub mod energy_supply;
pub mod towers_range;

use bevy::render::render_resource::ShaderType;

use crate::prelude::*;

pub struct OverlaysPlugin;
impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            towers_range::TowersRangeOverlayPlugin,
            energy_supply::EnergySupplyOverlayPlugin,
            emissions::EmissionsPlugin,
        ));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, ShaderType, Default)]
struct UniformGridData {
    grid_width: u32,
    grid_height: u32,
}