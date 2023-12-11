pub mod explosions;
pub mod common;

use bevy::prelude::*;

pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup, (
                explosions::load_assets_system,
            )
        );
        app.add_systems(
            Update, (
                common::animate_sprite_system,
                explosions::remove_explosions_system,
            )
        );
    }
}
