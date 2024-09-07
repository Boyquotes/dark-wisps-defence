use crate::prelude::*;
use crate::grids::base::GridVersion;
use crate::grids::common::{GridCoords, GridType};

pub mod prelude {
    pub use super::*;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameMode {
    Game,
    Editor,
    Manu,
}

#[derive(Resource)]
pub struct GameConfig {
    pub mode: GameMode,
}

pub fn is_editor_mode(config: Res<GameConfig>) -> bool {
    config.mode == GameMode::Editor
}

pub fn is_game_mode(config: Res<GameConfig>) -> bool {
    true || config.mode == GameMode::Game
}

#[derive(Debug, Clone, Copy)]
pub struct LazyEntity(Entity);
impl LazyEntity {
    pub fn get(&mut self, commands: &mut Commands) -> Entity {
        if self.0 == Entity::PLACEHOLDER {
            self.0 = commands.spawn_empty().id();
        }
        self.0
    }
}
impl Default for LazyEntity {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum TargetType {
    #[default]
    None,
    Field{coords: GridCoords, grid_version: GridVersion},
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
    Z_PROJECTILE,
    Z_AERIAL_UNIT
);