use bevy::prelude::*;
use crate::buildings::common::BuildingId;

#[derive(Resource, Default, Clone, Debug)]
pub enum UiInteractionState {
    #[default]
    Free, // No interaction
    PlaceGridObject,
    DisplayBuildingInfo(BuildingId),
}

pub fn keyboard_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_interaction_state: ResMut<UiInteractionState>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        *ui_interaction_state = UiInteractionState::Free;
        return;
    }
}