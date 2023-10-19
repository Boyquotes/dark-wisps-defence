pub mod grid_display;
pub mod interaction_state;
pub mod grid_object_placer;

use bevy::prelude::*;
use crate::ui::interaction_state::UiInteractionState;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiConfig::default());
        app.insert_resource(UiInteractionState::default());
        app.add_systems(Startup, grid_object_placer::create_grid_object_placer_system);
        app.add_systems(Update, (
            grid_display::show_hide_grid_system,
            grid_display::draw_grid_system,
        ));
        app.add_systems(Update, (
            grid_object_placer::update_grid_object_placer_system,
            grid_object_placer::on_click_initiate_grid_object_placer_system,
        ));

    }
}

#[derive(Resource, Default)]
pub struct UiConfig {
    pub show_grid: bool,
}