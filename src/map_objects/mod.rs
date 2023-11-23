pub mod walls;
pub mod dark_ore;

use bevy::prelude::*;


pub struct MapObjectsPlugin;
impl Plugin for MapObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            walls::onclick_spawn_system,
            walls::color_rotation_system,
            dark_ore::onclick_spawn_system,
        ));
    }
}
