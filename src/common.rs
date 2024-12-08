use crate::prelude::*;

pub mod prelude {
    pub use super::*;
}

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>();
    }
}

#[derive(Default, Clone, Debug, States, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Running,
    Paused,
}

pub enum UpgradeType {
    AttackSpeed,
    Range,
    Damage,
    Health
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