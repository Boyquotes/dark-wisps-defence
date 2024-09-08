pub mod resources;
pub mod almanach;
pub mod objectives;

use crate::prelude::*;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                objectives::ObjectivesPlugin,
            ))
            .insert_resource(almanach::Almanach::default())
            .insert_resource(objectives::ObjectivesCheckInactiveFlag::default())
            .insert_resource(resources::DarkOreStock::default());
    }
}