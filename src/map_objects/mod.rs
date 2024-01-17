pub mod walls;
pub mod dark_ore;
pub mod quantum_field;

use bevy::prelude::*;


pub struct MapObjectsPlugin;
impl Plugin for MapObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            quantum_field::create_grid_placer_ui_for_quantum_field_system,
        ));
        app.add_systems(Update, (
            walls::onclick_spawn_system,
            walls::color_rotation_system,
            dark_ore::onclick_spawn_system,
            quantum_field::onclick_spawn_system,
            quantum_field::operate_arrows_for_grid_placer_ui_for_quantum_field_system,
        ));
    }
}
