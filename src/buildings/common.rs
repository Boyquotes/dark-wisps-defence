use crate::prelude::*;

#[derive(Event)]
pub struct BuildingDestroyedEvent(pub Entity);
impl Command for BuildingDestroyedEvent {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}