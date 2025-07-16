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
            ));

        let mut obstacle_grid = obstacles::ObstacleGrid::new_empty();
        obstacle_grid.resize_and_reset((100, 100));
        app.insert_resource(obstacle_grid);
        let mut wisps_grid = wisps::WispsGrid::new_empty();
        wisps_grid.resize_and_reset((100, 100));
        app.insert_resource(wisps_grid);
    }
}