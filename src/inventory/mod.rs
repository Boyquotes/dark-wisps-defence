pub mod resources;
pub mod almanach;
pub mod objectives;
pub mod stats;

use crate::prelude::*;

pub mod prelude {
    pub use super::resources::{ResourceType, EssenceType, EssenceContainer, Stock, Cost, StockChangedEvent};
    pub use super::almanach::Almanach;
}

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                almanach::AlmanachPlugin,
                objectives::ObjectivesPlugin,
                resources::ResourcesPlugin,
                stats::StatsPlugin,
            ));
    }
}