use std::time::Duration;

use crate::prelude::*;

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(TowerShootingTimer::on_attack_speed_change);
    }
}

#[derive(Component, Default)]
#[require(Upgrades)]
pub struct MarkerTower;


// Building sub-parts markers
#[derive(Component)]
pub struct MarkerTowerRotationalTop(pub Entity);

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
        timer.0.set_duration(Duration::from_secs_f32(attack_speed.0));
        // Set to fire right away if it's first shot ever. This is not fault-proof solution as if it happens the AttackSpeed occurs exactly at 0. we can get double-shot.
        if timer.0.elapsed_secs() == 0. {
            timer.0.set_elapsed(Duration::from_secs_f32(attack_speed.0));
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