pub mod explosions;
pub mod common;
pub mod wisp_attack;

use crate::prelude::*;

pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                explosions::ExplosionPlugin,
                wisp_attack::WispAttackEffectPlugin,
            ))
            .add_systems(
            Update, (
                common::animate_sprite_system,
            ));
    }
}
