pub mod components;
pub mod spawning;
pub mod systems;

use crate::prelude::*;

pub struct WispsPlugin;
impl Plugin for WispsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<spawning::BuilderWisp>()
            .add_systems(PostUpdate, (
                spawning::BuilderWisp::spawn_system,
            ))
            .add_systems(Update, (
                systems::move_wisps,
                systems::target_wisps,
                systems::collide_wisps,
                systems::remove_dead_wisps,
                systems::spawn_wisps.run_if(is_game_mode)
            ));
    }
}