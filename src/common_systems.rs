use bevy::prelude::*;
use crate::common_components::ColorPulsation;


pub fn pulsate_sprites_system(
    mut sprites: Query<(&mut Sprite, &mut ColorPulsation)>,
    time: Res<Time>,
) {
    for (mut sprite, mut color_pulsation) in sprites.iter_mut() {
        color_pulsation.pulsate_sprite(&mut sprite, time.delta_seconds());
    }
}