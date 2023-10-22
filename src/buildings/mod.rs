pub mod components;
pub mod main_base;
pub mod common;
pub mod tower_blaster;

use bevy::prelude::*;

pub struct BuildingsPlugin;
impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tower_blaster::onclick_spawn_system,
                tower_blaster::shooting_system,
                tower_blaster::targeting_system
            )
        );
    }
}
