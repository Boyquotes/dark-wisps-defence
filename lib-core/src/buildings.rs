use crate::lib_prelude::*;

pub mod buildings_prelude {
    pub use super::*;
}

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

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum TowerType {
    Blaster,
    Cannon,
    RocketLauncher,
    Emitter,
}

#[derive(Component)]
pub struct MainBase;

#[derive(Component)]
pub struct EnergyRelay;

#[derive(Component)]
pub struct MiningComplex {
    pub ore_entities_in_range: Vec<Entity>,
}

#[derive(Component)]
pub struct ExplorationCenter;

#[derive(Component)]
pub struct TowerBlaster;

#[derive(Component)]
pub struct TowerCannon;

#[derive(Component)]
pub struct TowerRocketLauncher;

#[derive(Component)]
pub struct TowerEmitter;
