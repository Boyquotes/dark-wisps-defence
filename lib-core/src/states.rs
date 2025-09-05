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
            .init_state::<MapLoadingStage>()
            .add_systems(PreUpdate, (
                UiInteraction::on_escape.run_if(input_just_pressed(KeyCode::Escape)),
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
    Loading,
}
impl GameState {
    fn pause_resume_game(
        current_game_state: Res<State<GameState>>,
        mut next_game_state: ResMut<NextState<GameState>>
    ) {
        match current_game_state.get() {
            GameState::Paused => next_game_state.set(GameState::Running),
            GameState::Running => next_game_state.set(GameState::Paused),
            GameState::Loading => {}
        }
    }
}

#[derive(Default, Clone, Debug, States, PartialEq, Eq, Hash)]
pub enum UiInteraction {
    #[default]
    Free, // No interaction
    MainMenu,
    PlaceGridObject,
    DisplayInfoPanel,
}
impl UiInteraction {
    // On ESC: if UI is free, open Main Menu; otherwise, return to Free
    fn on_escape(
        current_ui_state: Res<State<UiInteraction>>, 
        mut next_ui_state: ResMut<NextState<UiInteraction>>
    ) {
        match current_ui_state.get() {
            UiInteraction::Free => next_ui_state.set(UiInteraction::MainMenu),
            _ => next_ui_state.set(UiInteraction::Free),
        }
    }
}

#[derive(Default, Clone, Debug, States, PartialEq, Eq, Hash)]
pub enum MapLoadingStage {
    #[default]
    Init,
    LoadMapFile,
    DespawnExisting, // If there is any data of current game, remove it
    ResetGridsAndResources,
    ApplyMap,
    Loaded,
}
impl MapLoadingStage {
    pub fn next(&self) -> Self {
        match self {
            MapLoadingStage::Init => MapLoadingStage::LoadMapFile,
            MapLoadingStage::LoadMapFile => MapLoadingStage::DespawnExisting,
            MapLoadingStage::DespawnExisting => MapLoadingStage::ResetGridsAndResources,
            MapLoadingStage::ResetGridsAndResources => MapLoadingStage::ApplyMap,
            MapLoadingStage::ApplyMap => MapLoadingStage::Loaded,
            MapLoadingStage::Loaded => MapLoadingStage::Loaded,
        }
    }
}