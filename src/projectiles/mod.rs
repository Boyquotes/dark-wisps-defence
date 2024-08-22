pub mod laser_dart;
pub mod components;
pub mod cannonball;
pub mod rocket;

use bevy::prelude::*;

pub struct ProjectilesPlugin;
impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                laser_dart::LaserDartPlugin,
                cannonball::CannonballPlugin,
            ))
            .add_systems(
            Update,
            (
                rocket::rocket_move_system,
                rocket::rocket_hit_system,
                rocket::exhaust_blinking_system,
            )
        );

    }
}
