pub mod laser_dart;
pub mod components;
pub mod cannonball;
pub mod rocket;

use bevy::prelude::*;

pub struct ProjectilesPlugin;
impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                laser_dart::laser_dart_move_system,
                laser_dart::laser_dart_hit_system,
                cannonball::cannonball_move_system,
                cannonball::cannonball_hit_system,
                rocket::rocket_move_system,
                rocket::rocket_hit_system,
            )
        );

    }
}
