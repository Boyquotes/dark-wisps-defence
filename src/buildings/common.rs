use crate::prelude::*;
use serde::{Deserialize, Serialize};
use crate::utils::id::Id;

use super::prelude::BuildingType;

pub type BuildingId = Id<BuildingType, Entity>;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum TowerType {
    Blaster,
    Cannon,
    RocketLauncher,
    Emitter,
}
