use crate::prelude::*;

// Building sub-parts markers
#[derive(Component)]
pub struct MarkerTowerRotationalTop(pub Entity);


#[derive(Component)]
pub struct TowerTopRotation {
    pub speed: f32, // in radians per second
    pub current_angle: f32,
}

#[derive(Message)]
pub struct BuildingDestroyedEvent(pub Entity);
impl Command for BuildingDestroyedEvent {
    fn apply(self, world: &mut World) {
        world.write_message(self);
    }
}