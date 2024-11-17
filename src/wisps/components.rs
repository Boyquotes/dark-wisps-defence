use crate::prelude::*;
use crate::grids::base::GridVersion;
use crate::utils::id::Id;

pub type WispEntity = Id<Wisp, Entity>;

#[derive(Component, Copy, Clone)]
pub enum WispType {
    Fire,
    Water,
    Light,
    Electric,
}
impl WispType {
    pub fn random() -> Self {
        let mut rng = nanorand::tls_rng();
        match rng.generate_range(1..=4) {  // Nano-rand is off by 1!
            0 => WispType::Fire,
            1 => WispType::Water,
            2 => WispType::Light,
            3 => WispType::Electric,
            _ => unreachable!(),
        }
    }
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

#[derive(Component)]
pub struct WispAttackRange(pub usize);