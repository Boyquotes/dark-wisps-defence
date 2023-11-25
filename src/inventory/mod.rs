pub mod resources;
mod almanach;

use bevy::prelude::*;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(resources::DarkOreStock::default());
        app.insert_resource(almanach::Almanach::default());
    }
}