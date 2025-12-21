use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};

use crate::prelude::*;

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, EnumIter, AsRefStr)]
pub enum WispType {
    Fire,
    Water,
    Light,
    Electric,
}

#[derive(Component)]
pub struct WispFireType;
#[derive(Component)]
pub struct WispWaterType;
#[derive(Component)]
pub struct WispLightType;
#[derive(Component)]
pub struct WispElectricType;


#[derive(Component, Debug, Default, PartialEq)]
#[require(WispState, WispChargeAttack, GridPath, MovementSpeed, AttackRange, MaxHealth, MapBound)]
pub struct Wisp;
#[derive(Component, Default)]
pub enum WispState {
    #[default]
    JustSpawned,
    NeedTarget,
    MovingToTarget,
    Attacking,
    Stranded(GridVersion), // No target available, waiting for change in obstacle grid
}

#[derive(Component, Default)]
pub enum WispChargeAttack {
    #[default]
    Charge,
    Backoff,
}