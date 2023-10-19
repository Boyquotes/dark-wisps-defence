pub mod obstacles;
pub mod common;
pub mod base_grid;

use bevy::prelude::*;

pub struct GridsPlugin;
impl Plugin for GridsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, obstacles::draw_grid_system);
        let mut grid = obstacles::ObstacleGrid::new_empty();
        grid.resize_and_reset(100, 100);
        app.insert_resource(grid);
        app.insert_resource(base_grid::GridBase::<i32>::new_empty());
    }
}