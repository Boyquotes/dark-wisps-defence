pub mod walls;

use bevy::prelude::*;
use crate::map_objects::walls::onclick_wall_spawn_system;


pub struct MapObjectsPlugin;
impl Plugin for MapObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, onclick_wall_spawn_system);
    }
}
