pub mod grid_display;
pub mod interaction_state;
pub mod grid_object_placer;
pub mod display_building_info;
pub mod badges;
mod construction_menu;
pub mod common;

use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiConfig::default());
        app.insert_resource(interaction_state::UiInteractionState::default());
        app.insert_resource(grid_object_placer::GridObjectPlacerRequest::default());
        app.add_systems(Startup, (
            badges::initialize_badges_system,
            construction_menu::initialize_construction_menu_system,
            grid_object_placer::create_grid_object_placer_system
        ));
        app.add_systems(PreUpdate, (
            common::mouse_release_system,
            grid_object_placer::keyboard_input_system,
            interaction_state::keyboard_input_system,
        ));
        app.add_systems(Update, (
            badges::sync_dark_ore_badge_system,
            construction_menu::menu_activation_system,
            construction_menu::construct_building_on_click_system,
            display_building_info::on_click_building_display_info_system,
            display_building_info::display_building_info_system,
            grid_display::show_hide_grid_system,
            grid_display::draw_grid_system,
            grid_object_placer::update_grid_object_placer_system,
            grid_object_placer::on_request_grid_object_placer_system,
        ));

    }
}

#[derive(Resource, Default)]
pub struct UiConfig {
    pub show_grid: bool,
}