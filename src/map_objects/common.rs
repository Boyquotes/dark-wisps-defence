use crate::prelude::*;

// Marks given expedition zone as valid target to send drones to
#[derive(Component, Default)]
pub struct ExpeditionTargetMarker;

#[derive(Component, Default)]
pub struct ExpeditionZone {
    pub expeditions_arrived: u32, // How many expeditions have arrived but was not yet processed
}