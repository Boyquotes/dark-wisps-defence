use bevy::prelude::*;


#[derive(Component, Default)]
pub struct AdvancedInteraction {
    pub was_just_released: bool,
}

pub fn mouse_release_system(
    mouse_button_input: Res<Input<MouseButton>>,
    mut advanced_interaction: Query<(&Interaction, &mut AdvancedInteraction), Changed<Interaction>>,
) {
    let was_mouse_just_released = mouse_button_input.just_released(MouseButton::Left);
    for (interaction, mut advanced_interaction) in advanced_interaction.iter_mut() {
        advanced_interaction.was_just_released = was_mouse_just_released && matches!(interaction, Interaction::Hovered);
    }
}