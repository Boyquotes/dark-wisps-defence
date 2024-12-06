use bevy::input::common_conditions::input_just_pressed;

use crate::prelude::*;
use crate::common_components::ColorPulsation;

pub struct CommonSystemsPlugin;
impl Plugin for CommonSystemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                pulsate_sprites_system, 
                pause_resume_game_system.run_if(input_just_pressed(KeyCode::Space)),
            ));
    }
}


pub fn pulsate_sprites_system(
    mut sprites: Query<(&mut Sprite, &mut ColorPulsation)>,
    time: Res<Time>,
) {
    for (mut sprite, mut color_pulsation) in sprites.iter_mut() {
        color_pulsation.pulsate_sprite(&mut sprite, time.delta_secs());
    }
}

pub fn pause_resume_game_system(
    current_game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>
) {
    match current_game_state.get() {
        GameState::Paused => next_game_state.set(GameState::Running),
        GameState::Running => next_game_state.set(GameState::Paused),
    }
}

