mod construction_menu;
mod objectives_panel;
mod badges;
pub mod common;
pub mod display_info_panel;
pub mod grid_display;
pub mod grid_object_placer;

use crate::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                badges::BadgesPlugin,
                display_info_panel::DisplayInfoPanelPlugin,
                grid_object_placer::GridObjectPlacerPlugin,
                objectives_panel::ObjectivesPanelPlugin,
                construction_menu::ConstructionMenuPlugin,
            ))
            .insert_resource(UiConfig::default())
            .add_systems(Update, (
                grid_display::show_hide_grid_system,
                grid_display::draw_grid_system,
            ));

    }
}

#[derive(Resource, Default)]
pub struct UiConfig {
    pub show_grid: bool,
}