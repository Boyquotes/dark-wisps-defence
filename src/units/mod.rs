pub mod expedition_drone;

use crate::prelude::*;

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            expedition_drone::move_expedition_drone_system,
        ));
    }
}
