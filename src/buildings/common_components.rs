use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::grids::common::GridImprint;

#[derive(Component)]
pub struct Building {
    pub grid_imprint: GridImprint,
    pub building_type: BuildingType,
}

// Building type markers
#[derive(Component)]
pub struct MarkerMainBase;

#[derive(Component)]
pub struct MarkerTower;

#[derive(Component)]
pub struct TowerShootingTimer(pub Timer);
