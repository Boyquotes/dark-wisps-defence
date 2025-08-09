use crate::lib_prelude::*;

pub mod common_prelude {
    pub use super::*;
}

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                ColorPulsation::pulsate_sprites_system,
            ))
            .add_observer(ZDepth::on_insert)
            .add_observer(MaxHealth::on_insert);
    }
}

// Simple property trait for single value objects. Useful in generic contexts.
pub trait Property {
    fn get(&self) -> f32;
    fn set(&mut self, value: f32);
    fn new(value: f32) -> Self;
}


#[derive(Component, Default)]
pub struct Health {
    current: f32,
    max: f32, // A helper, source of truth is in MaxHealth component
}
impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    pub fn get_current(&self) -> f32 {
        self.current
    }
    pub fn get_max(&self) -> f32 {
        self.max
    }
    pub fn get_percent(&self) -> f32 {
        self.current / self.max
    }
    pub fn decrease(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.);
    }
    pub fn is_dead(&self) -> bool {
        self.current <= 0.
    }
}

#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
#[require(Health)]
pub struct MaxHealth(pub f32);
impl MaxHealth {
    fn on_insert(
        trigger: Trigger<OnInsert, MaxHealth>,
        mut healths: Query<(&mut Health, &MaxHealth)>,
    ) {
        let Ok((mut health, max_health)) = healths.get_mut(trigger.target()) else { return; };
        if health.current == 0. {
            health.current = max_health.0;
        }
        health.max = max_health.0;
    }
}
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct MovementSpeed(pub f32);
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct AttackSpeed(pub f32);
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct AttackDamage(pub f32);
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct AttackRange(pub f32);
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct EnergySupplyRange(pub f32);


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

    fn pulsate_sprites_system(
        time: Res<Time>,
        mut sprites: Query<(&mut Sprite, &mut ColorPulsation)>,
    ) {
        for (mut sprite, mut color_pulsation) in sprites.iter_mut() {
            let delta_time = time.delta_secs();
            match &mut sprite.color {
                Color::Hsla(Hsla {lightness, .. }) => {
                    if color_pulsation.is_increasing && *lightness > color_pulsation.max_brightness {
                        color_pulsation.is_increasing = false;
                    } else if !color_pulsation.is_increasing && *lightness < color_pulsation.min_brightness {
                        color_pulsation.is_increasing = true;
                    }
                    *lightness += delta_time * color_pulsation.delta_change * if color_pulsation.is_increasing { 1. } else { -1. }
                }
                _ => {}
            }
        }
    }
}

#[derive(Component)]
#[component(immutable)]
#[require(Transform)]
pub struct ZDepth(pub f32);
impl ZDepth {
    fn on_insert(
        trigger: Trigger<OnInsert, ZDepth>,
        mut transforms: Query<(&mut Transform, &ZDepth)>,
    ) {
        let entity = trigger.target();
        let Ok((mut transform, z_depth)) = transforms.get_mut(entity) else { return; };
        transform.translation.z = z_depth.0;
    }
}
impl From<f32> for ZDepth {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

macro_rules! define_z_indexes {
    // Internal macro to handle incrementing the counter
    (@internal $counter:expr, $name:ident) => {
        pub const $name: f32 = $counter;
    };
    (@internal $counter:expr, $name:ident, $($rest:ident),+) => {
        pub const $name: f32 = $counter;
        define_z_indexes!(@internal $counter + 0.001, $($rest),+);
    };
    // Public-facing macro interface
    ($($name:ident),+) => {
        define_z_indexes!(@internal 0.001, $($name),+);
    };
}

define_z_indexes!(
    Z_OBSTACLE,
    Z_OVERLAY_ENERGY_SUPPLY,
    Z_BUILDING,
    Z_WISP,
    Z_GROUND_EFFECT,
    Z_TOWER_TOP,
    Z_MAP_UI,
    Z_AERIAL_UNIT,
    Z_PROJECTILE_UNDER,
    Z_PROJECTILE
);