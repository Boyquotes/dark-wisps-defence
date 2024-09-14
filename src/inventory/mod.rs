pub mod resources;
pub mod almanach;
pub mod objectives;
pub mod stats;

use crate::prelude::*;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                objectives::ObjectivesPlugin,
                stats::StatsPlugin,
            ))
            .insert_resource(almanach::Almanach::default())
            .insert_resource(objectives::ObjectivesReassesInactiveFlag::default())
            .insert_resource(resources::DarkOreStock::default());
    }
}