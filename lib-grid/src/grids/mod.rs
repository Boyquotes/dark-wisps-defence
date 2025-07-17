pub mod obstacles;
pub mod common;
pub mod base;
pub mod wisps;
pub mod visited;
pub mod emissions;
pub mod energy_supply;

use crate::lib_prelude::*;

pub mod prelude {
    pub use super::common::prelude::*;
}

pub struct GridsPlugin;
impl Plugin for GridsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                emissions::EmissionsPlugin,
                energy_supply::EnergySupplyPlugin,
                obstacles::ObstaclesGridPlugin,
                wisps::WispsGridPlugin,
            ));
    }
}