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

#[derive(Component, Clone, Debug, Default)]
#[require(AutoGridTransformSync, ZDepth = ZDepth(Z_BUILDING))]
pub struct Building;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::MainBase)]
pub struct MainBase;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::EnergyRelay)]
pub struct EnergyRelay;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::MiningComplex)]
pub struct MiningComplex {
    pub ore_entities_in_range: Vec<Entity>,
}

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::ExplorationCenter)]
pub struct ExplorationCenter;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::Tower(TowerType::Blaster))]
pub struct TowerBlaster;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::Tower(TowerType::Cannon))]
pub struct TowerCannon;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::Tower(TowerType::RocketLauncher))]
pub struct TowerRocketLauncher;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::Tower(TowerType::Emitter))]
pub struct TowerEmitter;


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