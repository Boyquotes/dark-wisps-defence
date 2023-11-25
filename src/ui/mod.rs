pub mod grid_display;
pub mod interaction_state;
pub mod grid_object_placer;
pub mod display_building_info;
pub mod badges;

use bevy::prelude::*;
use crate::ui::interaction_state::UiInteractionState;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiConfig::default());
        app.insert_resource(UiInteractionState::default());
        app.add_systems(Startup, (
            badges::initialize_badges_system,
            grid_object_placer::create_grid_object_placer_system
        ));
        app.add_systems(Update, (
            badges::sync_dark_ore_badge_system,
            display_building_info::on_click_building_display_info_system,
            display_building_info::display_building_info_system,
            grid_display::show_hide_grid_system,
            grid_display::draw_grid_system,
            grid_object_placer::update_grid_object_placer_system,
            grid_object_placer::on_click_initiate_grid_object_placer_system,
        ));

    }
}

#[derive(Resource, Default)]
pub struct UiConfig {
    pub show_grid: bool,
}