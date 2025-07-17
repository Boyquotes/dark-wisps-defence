pub mod emissions;
pub mod energy_supply;

use crate::prelude::*;

pub struct OverlaysPlugin;
impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            energy_supply::EnergySupplyOverlayPlugin,
            emissions::EmissionsPlugin,
        ));
    }
}
