pub mod common_components;
pub mod main_base;
pub mod common;
pub mod tower_blaster;
pub mod tower_cannon;
pub mod common_systems;

use bevy::prelude::*;

pub struct BuildingsPlugin;
impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, common_systems::tick_shooting_timers_system);
        app.add_systems(
            Update,
            (
                common_systems::onclick_building_spawn_system,
                tower_blaster::shooting_system,
                tower_blaster::targeting_system,
                tower_cannon::shooting_system,
                tower_cannon::targeting_system,
            )
        );
    }
}
