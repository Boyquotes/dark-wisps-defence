use crate::lib_prelude::*;


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
