pub mod grid_display;
pub mod interaction_state;
pub mod grid_object_placer;

use bevy::prelude::*;
use crate::ui::grid_object_placer::{create_grid_object_placer_system, on_click_initiate_grid_object_placer_system, update_grid_object_placer_system};
use crate::ui::grid_display::show_hide_grid_system;
use crate::ui::interaction_state::UiInteractionState;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiConfig::default());
        app.insert_resource(UiInteractionState::default());
        app.add_systems(Startup, create_grid_object_placer_system);
        app.add_systems(Update, show_hide_grid_system);
        app.add_systems(Update, (
            update_grid_object_placer_system,
            on_click_initiate_grid_object_placer_system,
        ));

    }
}

#[derive(Resource, Default)]
pub struct UiConfig {
    pub show_grid: bool,
}