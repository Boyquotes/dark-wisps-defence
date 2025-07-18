use bevy::input::common_conditions::input_just_pressed;

use crate::lib_prelude::*;

pub mod states_prelude {
    pub use super::*;
}

pub struct StatesPlugin;
impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .init_state::<UiInteraction>()
            .add_systems(PreUpdate, (
                UiInteraction::free.run_if(input_just_pressed(KeyCode::Escape)),
            ))
            .add_systems(Update, (
                GameState::pause_resume_game.run_if(input_just_pressed(KeyCode::Space)),
            ));
    }
}

#[derive(Default, Clone, Debug, States, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Running,
    Paused,
}
impl GameState {
    fn pause_resume_game(
        current_game_state: Res<State<GameState>>,
        mut next_game_state: ResMut<NextState<GameState>>
    ) {
        match current_game_state.get() {
            GameState::Paused => next_game_state.set(GameState::Running),
            GameState::Running => next_game_state.set(GameState::Paused),
        }
    }
}

#[derive(Default, Clone, Debug, States, PartialEq, Eq, Hash)]
pub enum UiInteraction {
    #[default]
    Free, // No interaction
    PlaceGridObject,
    DisplayInfoPanel,
}
impl UiInteraction {
    fn free(mut ui_interaction_state: ResMut<NextState<UiInteraction>>) {
        ui_interaction_state.set(UiInteraction::Free);
    }
}