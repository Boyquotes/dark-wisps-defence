use crate::prelude::*;

pub mod prelude {
    pub use super::*;
}

#[derive(Component, Default)]
pub struct Health {
    current: i32,
    max: i32,
}
impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }
    pub fn get_current(&self) -> i32 {
        self.current
    }
    pub fn get_max(&self) -> i32 {
        self.max
    }
    pub fn get_percent(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
    pub fn decrease(&mut self, amount: i32) {
        self.current = std::cmp::max(self.current - amount, 0);
    }
    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

#[derive(Component)]
pub struct EssencesContainer(pub Vec<EssenceContainer>);
impl From<EssenceContainer> for EssencesContainer {
    fn from(essence_container: EssenceContainer) -> Self {
        Self(vec![essence_container])
    }
}

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component, Default)]
pub struct ColorPulsation {
    min_brightness: f32,
    max_brightness: f32,
    duration: f32,
    is_increasing: bool,
    delta_change: f32,
}
impl ColorPulsation {
    pub fn new(min_brightness: f32, max_brightness: f32, duration: f32) -> Self {
        let mut color_pulsation = ColorPulsation::default();
        color_pulsation.update_parameters(min_brightness, max_brightness, duration);
        color_pulsation
    }
    pub fn update_parameters(&mut self, min_brightness: f32, max_brightness: f32, duration: f32) {
        assert!(min_brightness < max_brightness, "min_brightness must be less than max_brightness");
        self.min_brightness = min_brightness;
        self.max_brightness = max_brightness;
        self.duration = duration;
        self.delta_change = (max_brightness - min_brightness) / duration;
    }
    pub fn pulsate_sprite(&mut self, sprite: &mut Sprite, delta_time: f32) {
        match &mut sprite.color {
            Color::Hsla(Hsla {lightness, .. }) => {
                if self.is_increasing && *lightness > self.max_brightness {
                    self.is_increasing = false;
                } else if !self.is_increasing && *lightness < self.min_brightness {
                    self.is_increasing = true;
                }
                *lightness += delta_time * self.delta_change * if self.is_increasing { 1. } else { -1. }
            }
            _ => {}
        }
    }
}