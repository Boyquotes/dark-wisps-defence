use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::utils::id::Id;

pub type BuildingId = Id<BuildingType, Entity>;
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum BuildingType {
    EnergyRelay,
    MainBase,
    Tower(TowerType),
}
impl BuildingType {
    pub fn is_energy_rich(&self) -> bool {
        matches!(self, BuildingType::MainBase | BuildingType::EnergyRelay)
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TowerType {
    Blaster,
    Cannon,
}
