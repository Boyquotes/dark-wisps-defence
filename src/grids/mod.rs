pub mod obstacles;
pub mod common;
pub mod base;

use bevy::prelude::*;

pub struct GridsPlugin;
impl Plugin for GridsPlugin {
    fn build(&self, app: &mut App) {
        let mut grid = obstacles::ObstacleGrid::new_empty();
        grid.resize_and_reset(100, 100);
        app.insert_resource(grid);
    }
}