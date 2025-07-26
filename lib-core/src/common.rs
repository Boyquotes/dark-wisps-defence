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
            .add_observer(ZDepth::on_insert);
    }
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

#[derive(Component, Default, Clone)]
#[component(immutable)]
pub struct Speed(pub f32);
#[derive(Component, Default, Clone)]
pub struct AttackSpeed(pub f32);
#[derive(Component, Default, Clone)]
#[component(immutable)]
pub struct AttackDamage(pub f32);
#[derive(Component, Default, Clone)]
#[component(immutable)]
pub struct AttackRange(pub usize);


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



#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpgradeType {
    AttackSpeed,
    AttackRange,
    AttackDamage,
    Health
}
impl std::fmt::Display for UpgradeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Component, Default)]
pub struct Upgrades(pub HashMap<UpgradeType, usize>);
impl Upgrades {
    pub fn total(&self) -> usize {
        self.0.values().sum()
    }
    // fn on_upgrade_level_up(
    //     trigger: Trigger<UpgradeLevelUp>,
    //     mut commands: Commands,
    //     almanach: Res<Almanach>,
    //     mut upgrades: Query<(&mut Upgrades, &BuildingType, AnyOf<(&AttackSpeed, &AttackRange, &AttackDamage, &Health)>)>,
    // ) {
    //     let entity = trigger.target();
    //     let Ok((mut upgrades, building_type, (maybe_attack_speed, maybe_attack_range, maybe_attack_damage, maybe_health))) = upgrades.get_mut(entity) else { return; };
    //     let upgrade_type = trigger.0;
    //     let current_upgrade_level = *upgrades.0.get(&upgrade_type).unwrap_or(&0);
    //     let next_upgrade_level = current_upgrade_level + 1;
    //     upgrades.0.insert(upgrade_type, next_upgrade_level);
    //     commands.entity(entity).insert(
    //         match upgrade_type {
    //             UpgradeType::AttackRange => {
    //                 AttackRange(maybe_attack_range + almanach)
    //             }
    //             UpgradeType::AttackDamage => {
    //                 if let Some(attack_damage) = maybe_attack_damage {
    //                     attack_damage.0 += 1.;
    //                 }
    //             }
    //             UpgradeType::Health => {
    //                 if let Some(health) = maybe_health {
    //                     health.0 += 1;
    //                 }
    //             }
    //         }
    //     );
    // }
}

#[derive(Event)]
pub struct UpgradeLevelUp(pub UpgradeType);

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
    Z_PROJECTILE_UNDER,
    Z_PROJECTILE,
    Z_AERIAL_UNIT
);