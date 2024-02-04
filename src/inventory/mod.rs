pub mod resources;
pub mod almanach;
pub mod objectives;

use bevy::prelude::*;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(almanach::Almanach::default());
        app.insert_resource(objectives::ObjectivesCheckInactiveFlag::default());
        app.insert_resource(resources::DarkOreStock::default());
    }
}