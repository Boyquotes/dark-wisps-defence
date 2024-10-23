use crate::prelude::*;

pub struct UiCommonPlugin;
impl Plugin for UiCommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<UiInteraction>()
            .add_systems(PreUpdate, (
                keyboard_input_system,
                mouse_release_system,
            ));
    }
}

#[derive(Default, Clone, Debug, States, PartialEq, Eq, Hash)]
pub enum UiInteraction {
    #[default]
    Free, // No interaction
    PlaceGridObject,
    DisplayInfoPanel,
}

fn keyboard_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        ui_interaction_state.set(UiInteraction::Free);
    }
}


#[derive(Component, Default)]
pub struct AdvancedInteraction {
    pub was_just_released: bool,
}

fn mouse_release_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut advanced_interaction: Query<(&Interaction, &mut AdvancedInteraction)>,
) {
    let was_mouse_just_released = mouse_button_input.just_released(MouseButton::Left);
    for (interaction, mut advanced_interaction) in advanced_interaction.iter_mut() {
        advanced_interaction.was_just_released = was_mouse_just_released && matches!(interaction, Interaction::Hovered);
    }
}