pub mod common_components;
pub mod main_base;
pub mod common;
pub mod tower_blaster;
pub mod tower_cannon;
pub mod common_systems;
pub mod energy_relay;
pub mod tower_rocket_launcher;
pub mod mining_complex;
pub mod exploration_center;

use crate::prelude::*;

pub struct BuildingsPlugin;
impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, common_systems::tick_shooting_timers_system);
        app.add_systems(
            Update,
            (
                common_systems::onclick_building_spawn_system,
                common_systems::check_energy_supply_system,
                common_systems::targeting_system,
                common_systems::rotate_tower_top_system,
                common_systems::rotational_aiming_system,
                exploration_center::create_expedition_system,
                mining_complex::mine_ore_system,
                tower_blaster::shooting_system,
                tower_cannon::shooting_system,
                tower_rocket_launcher::shooting_system,
            )
        );
    }
}
