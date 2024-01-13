use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grids::common::GridImprint;
use crate::utils::id::Id;

pub type BuildingId = Id<BuildingType, Entity>;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum BuildingType {
    EnergyRelay,
    MainBase,
    Tower(TowerType),
    MiningComplex,
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
                }
            },
            BuildingType::MiningComplex => super::mining_complex::MINING_COMPLEX_GRID_IMPRINT,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum TowerType {
    Blaster,
    Cannon,
    RocketLauncher,
}
