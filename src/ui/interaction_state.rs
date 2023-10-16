use bevy::prelude::*;

#[derive(Resource, Default, Clone, Debug)]
pub enum UiInteractionState {
    #[default]
    Free, // No interaction
    PlaceGridObject,
}