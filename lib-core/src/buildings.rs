use std::time::Duration;

use crate::lib_prelude::*;

pub mod buildings_prelude {
    pub use super::*;
}

pub struct BuildingsPlugin;
impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(TowerShootingTimer::on_attack_speed_change);
    }
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
    /// EnergyRelay is considered a consumer as it cannot operate without energy supply
    pub fn is_energy_consumer(&self) -> bool {
        !matches!(self, BuildingType::MainBase)
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
#[require(AutoGridTransformSync, ZDepth = Z_BUILDING, MaxHealth, TechnicalState)]
pub struct Building;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::MainBase)]
pub struct MainBase;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::EnergyRelay)]
pub struct EnergyRelay;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::MiningComplex)]
pub struct MiningComplex;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::ExplorationCenter)]
pub struct ExplorationCenter;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::Tower(TowerType::Blaster), AttackRange, AttackSpeed, AttackDamage, TowerShootingTimer, TowerWispTarget)]
pub struct TowerBlaster;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::Tower(TowerType::Cannon), AttackRange, AttackSpeed, AttackDamage, TowerShootingTimer, TowerWispTarget)]
pub struct TowerCannon;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::Tower(TowerType::RocketLauncher), AttackRange, AttackSpeed, AttackDamage, TowerShootingTimer, TowerWispTarget)]
pub struct TowerRocketLauncher;

#[derive(Component)]
#[require(Building, BuildingType = BuildingType::Tower(TowerType::Emitter), AttackRange, AttackSpeed, AttackDamage, TowerShootingTimer, TowerWispTarget)]
pub struct TowerEmitter;


#[derive(Component, Default)]
pub struct TechnicalState {
    pub has_energy_supply: bool,
    pub has_power: bool,
    pub has_ore_fields: Option<bool>,
}
impl TechnicalState {
    pub fn is_operational(&self) -> bool {
        self.has_power && self.has_ore_fields.unwrap_or(true)
    }
}

#[derive(Component, Default)]
#[require(AttackSpeed)]
pub struct TowerShootingTimer(pub Timer);
impl TowerShootingTimer {
    fn on_attack_speed_change(
        trigger: Trigger<OnInsert, AttackSpeed>,
        mut timers: Query<(&mut TowerShootingTimer, &AttackSpeed)>
    ) {
        let entity = trigger.target();
        let Ok((mut timer, attack_speed)) = timers.get_mut(entity) else { return; };
        if attack_speed.0 == 0. { return; }
        timer.0.set_duration(Duration::from_secs_f32(1. / attack_speed.0));
        // Set to fire right away if it's first shot ever. This is not fault-proof solution as if it happens the AttackSpeed occurs exactly at 0. we can get double-shot.
        if timer.0.elapsed_secs() == 0. {
            timer.0.set_elapsed(Duration::from_secs_f32(1. / attack_speed.0));
        }
    }
}

#[derive(Component, Default)]
pub enum TowerWispTarget {
    #[default]
    SearchForNewTarget,
    Wisp(Entity),
    NoValidTargets(GridVersion),
}
