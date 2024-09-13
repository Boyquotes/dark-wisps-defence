use crate::prelude::*;
use crate::grids::base::GridVersion;
use crate::utils::id::Id;

pub type WispEntity = Id<Wisp, Entity>;


#[derive(Component, Debug, Default, PartialEq)]
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