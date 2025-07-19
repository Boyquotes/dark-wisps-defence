use bevy::app::{App, Plugin};

pub mod healthbar;
pub mod cost_indicator;
pub mod utils;

pub struct LibUiPlugin;
impl Plugin for LibUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                healthbar::HealthbarPlugin,
                cost_indicator::CostIndicatorPlugin,
                utils::UtilsPlugin,
            ));
    }
}

pub mod prelude {
    pub use crate::healthbar::Healthbar;
    pub use crate::cost_indicator::CostIndicator;
    pub use crate::utils::AdvancedInteraction;
}

pub mod lib_prelude {
    pub use bevy::prelude::*;

    pub use lib_core::prelude::*;
    pub use lib_inventory::prelude::*;
}