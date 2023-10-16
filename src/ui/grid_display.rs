use bevy::prelude::*;
use crate::ui::UiConfig;

pub fn show_hide_grid_system(mut ui_config: ResMut<UiConfig>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::G) {
        ui_config.show_grid = !ui_config.show_grid;
    }
}