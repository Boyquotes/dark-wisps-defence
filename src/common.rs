use bevy::prelude::*;
use crate::grids::base::GridVersion;
use crate::grids::common::{GridCoords, GridType};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GridDynamicObject {
    Wisp(Entity),
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum TargetType {
    #[default]
    None,
    Field{coords: GridCoords, grid_version: GridVersion},
    DynamicObject(GridDynamicObject),
    Unreachable{grid_type: GridType, grid_version: GridVersion},
}

impl TargetType {
    pub fn is_some(&self) -> bool {
        !matches!(self, TargetType::None) && !matches!(self, TargetType::Unreachable{..})
    }
    pub fn is_unreachable(&self) -> bool {
        matches!(self, TargetType::Unreachable{..})
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
    Z_BUILDING,
    Z_WISP,
    Z_GROUND_EFFECT,
    Z_TOWER_TOP,
    Z_PROJECTILE_UNDER,
    Z_PROJECTILE
);