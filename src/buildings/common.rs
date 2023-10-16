use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::utils::id::Id;

pub type BuildingId = Id<BuildingType, Entity>;
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum BuildingType {
    MainBase,
    Tower(TowerType),
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TowerType {
    Blaster,
}
