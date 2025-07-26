use std::time::Duration;

use crate::prelude::*;

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
    }
}

#[derive(Component, Default)]
#[require(Upgrades)]
pub struct MarkerTower;


// Building sub-parts markers
#[derive(Component)]
pub struct MarkerTowerRotationalTop(pub Entity);

#[derive(Component, Default)]
pub struct TowerShootingTimer(pub Timer);
impl TowerShootingTimer {
    pub fn from_seconds(seconds: f32) -> Self {
        let mut timer = Timer::from_seconds(seconds, TimerMode::Once);
        // Set it ready to fire right away
        timer.set_elapsed(Duration::from_secs_f32(seconds));
        Self(timer)
    }
}

#[derive(Component, Default)]
pub enum TowerWispTarget {
    #[default]
    SearchForNewTarget,
    Wisp(Entity),
    NoValidTargets(GridVersion),
}

#[derive(Component)]
pub struct TowerTopRotation {
    pub speed: f32, // in radians per second
    pub current_angle: f32,
}

#[derive(Event)]
pub struct BuildingDestroyedEvent(pub Entity);
impl Command for BuildingDestroyedEvent {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}