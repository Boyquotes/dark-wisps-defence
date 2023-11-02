use bevy::prelude::*;
use crate::buildings::common::BuildingId;

#[derive(Resource, Default, Clone, Debug)]
pub enum UiInteractionState {
    #[default]
    Free, // No interaction
    PlaceGridObject,
    DisplayBuildingInfo(BuildingId),
}