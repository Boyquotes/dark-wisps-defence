use std::time::Duration;
use bevy::prelude::*;
use crate::buildings::common::{BuildingId, BuildingType};
use crate::grids::base::GridVersion;
use crate::grids::common::GridImprint;
use crate::wisps::components::WispEntity;

#[derive(Component, Clone, Debug)]
pub struct Building {
    pub grid_imprint: GridImprint,
    pub building_type: BuildingType,
}

#[derive(Component, Default)]
pub struct TechnicalState {
    pub has_energy_supply: bool,
}
impl TechnicalState {
    pub fn is_operational(&self) -> bool {
        self.has_energy_supply
    }
}

// Building type markers
#[derive(Component)]
pub struct MarkerMainBase;
#[derive(Component)]
pub struct MarkerTower;
#[derive(Component)]
pub struct MarkerEnergyRelay;
#[derive(Component)]
pub struct MarkerMiningComplex;


// Building sub-parts markers
#[derive(Component)]
pub struct MarkerTowerRotationalTop(pub BuildingId);

#[derive(Component)]
pub struct TowerShootingTimer(pub Timer);
impl TowerShootingTimer {
    pub fn from_seconds(seconds: f32) -> Self {
        let mut timer = Timer::from_seconds(seconds, TimerMode::Once);
        // Set it ready to fire right away
        timer.set_elapsed(Duration::from_secs_f32(seconds));
        Self(timer)
    }
}

#[derive(Component, Default)]
pub enum TowerWispTarget {
    #[default]
    SearchForNewTarget,
    Wisp(WispEntity),
    NoValidTargets(GridVersion),
}

#[derive(Component)]
pub struct TowerRange(pub usize);

#[derive(Component)]
pub struct TowerTopRotation {
    pub speed: f32, // in radians per second
    pub current_angle: f32,
}