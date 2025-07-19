use crate::lib_prelude::*;


pub struct UtilsPlugin;
impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, (
                mouse_release_system,
            ));
    }
}


#[derive(Component, Default)]
#[require(Interaction)]
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


/// Recolor BackgroundColor with the given color on the specifed trigger.
/// Example use: `.observe(recolor_background_on::<Pointer<Out>>(Color::NONE))`
pub fn recolor_background_on<E>(color: Color) -> impl Fn(Trigger<E>, Query<&mut BackgroundColor>) {
    move |event, mut background_colors| {
        let Ok(mut background_color) = background_colors.get_mut(event.target()) else {
            return;
        };
        background_color.0 = color;
    }
}
