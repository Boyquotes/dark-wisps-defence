mod construction_menu;
mod objectives_panel;
mod badges;
pub mod common;
pub mod display_info_panel;
pub mod grid_display;
pub mod grid_object_placer;

use crate::prelude::*;

pub mod prelude {
    pub use super::common::{AdvancedInteraction, UiInteraction, Healthbar, HealthbarBundle};
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            common::UiCommonPlugin,
            display_info_panel::DisplayInfoPanelPlugin,
            grid_object_placer::GridObjectPlacerPlugin,
            objectives_panel::ObjectivesPanelPlugin,
        ));
        app.insert_resource(UiConfig::default());
        app.add_systems(Startup, (
            badges::initialize_badges_system,
            construction_menu::initialize_construction_menu_system,
        ));
        app.add_systems(Update, (
            badges::sync_dark_ore_badge_system,
            construction_menu::menu_activation_system,
            construction_menu::construct_building_on_click_system,
            grid_display::show_hide_grid_system,
            grid_display::draw_grid_system,
        ));

    }
}

#[derive(Resource, Default)]
pub struct UiConfig {
    pub show_grid: bool,
}