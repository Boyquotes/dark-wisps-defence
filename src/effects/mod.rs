pub mod explosions;
pub mod common;

use crate::prelude::*;

pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                explosions::ExplosionPlugin,
            ))
            .add_systems(
            Update, (
                common::animate_sprite_system,
            ));
    }
}
