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
                rocket::RocketPlugin,
            ));

    }
}
