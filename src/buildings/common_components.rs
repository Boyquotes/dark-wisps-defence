use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::prelude::*;
use crate::grids::base::GridVersion;
use crate::wisps::components::WispEntity;

#[derive(Component, Clone, Debug)]
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
    pub fn is_energy_rich(&self) -> bool {
        matches!(self, BuildingType::MainBase | BuildingType::EnergyRelay)
    }
    pub fn grid_imprint(&self) -> GridImprint {
        match self {
            BuildingType::EnergyRelay => super::energy_relay::ENERGY_RELAY_GRID_IMPRINT,
            BuildingType::MainBase => super::main_base::MAIN_BASE_GRID_IMPRINT,
            BuildingType::Tower(tower_type) => {
                match tower_type {
                    TowerType::Blaster => super::tower_blaster::TOWER_BLASTER_GRID_IMPRINT,
                    TowerType::Cannon => super::tower_cannon::TOWER_CANNON_GRID_IMPRINT,
                    TowerType::RocketLauncher => super::tower_rocket_launcher::TOWER_ROCKET_LAUNCHER_GRID_IMPRINT,
                    TowerType::Emitter => super::tower_emitter::TOWER_EMITTER_GRID_IMPRINT,
                }
            },
            BuildingType::MiningComplex => super::mining_complex::MINING_COMPLEX_GRID_IMPRINT,
            BuildingType::ExplorationCenter => super::exploration_center::EXPLORATION_CENTER_GRID_IMPRINT,
        }
    }
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

#[derive(Component)]
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