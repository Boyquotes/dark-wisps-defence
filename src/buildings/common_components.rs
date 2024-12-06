use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::prelude::*;
use crate::grids::base::GridVersion;
use crate::wisps::components::WispEntity;

#[derive(Component, Clone, Debug, Default)]
#[require(Level)]
pub struct Building;

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum BuildingType {
    EnergyRelay,
    MainBase,
    Tower(TowerType),
    MiningComplex,
    ExplorationCenter,
}
impl BuildingType {
    pub fn is_energy_supplier(&self) -> bool {
        matches!(self, BuildingType::MainBase | BuildingType::EnergyRelay)
    }
}

#[derive(Component, Default)]
pub struct TechnicalState {
    pub has_energy_supply: bool,
    pub has_ore_fields: Option<bool>,
}
impl TechnicalState {
    pub fn is_operational(&self) -> bool {
        self.has_energy_supply && self.has_ore_fields.unwrap_or(true)
    }
}

#[derive(Component, Default)]
#[require(Building)]
pub struct MarkerTower;


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