use crate::prelude::*;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum TowerType {
    Blaster,
    Cannon,
    RocketLauncher,
    Emitter,
}

#[derive(Event)]
pub struct BuildingDestroyedEvent(pub Entity);
impl Command for BuildingDestroyedEvent {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}