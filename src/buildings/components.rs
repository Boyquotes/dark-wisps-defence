use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::grid::GridImprint;

#[derive(Component)]
pub struct Building {
    pub grid_imprint: GridImprint,
    pub building_type: BuildingType,
}

// Building type markers
#[derive(Component)]
pub struct MarkerMainBase;