use crate::buildings::tower_blaster::{onclick_tower_blaster_spawn_system, tower_blaster_shooting_system};

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
                onclick_tower_blaster_spawn_system,
                tower_blaster_shooting_system,
            )
        );
    }
}
