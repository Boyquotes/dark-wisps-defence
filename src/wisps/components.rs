use crate::prelude::*;

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum WispType {
    Fire,
    Water,
    Light,
    Electric,
}

impl WispType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WispType::Fire => "Fire",
            WispType::Water => "Water",
            WispType::Light => "Light",
            WispType::Electric => "Electric",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Fire" => Some(WispType::Fire),
            "Water" => Some(WispType::Water),
            "Light" => Some(WispType::Light),
            "Electric" => Some(WispType::Electric),
            _ => None,
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