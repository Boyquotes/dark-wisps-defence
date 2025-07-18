use crate::prelude::*;
use crate::common_components::ColorPulsation;

pub struct CommonSystemsPlugin;
impl Plugin for CommonSystemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                pulsate_sprites_system, 
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