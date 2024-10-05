use std::collections::VecDeque;
use crate::prelude::*;
use crate::grids::base::GridVersion;

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

#[derive(Component, Default)]
pub struct GridPath {
    pub grid_version: GridVersion,
    pub path: VecDeque<GridCoords>,
}
impl GridPath {
    pub fn next_in_path(&self) -> Option<GridCoords> {
        self.path.front().copied()
    }
    pub fn remove_first(&mut self) {
        self.path.pop_front();
    }
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
    pub fn distance(&self) -> usize {
        self.path.len()
    }
    pub fn at_distance(&self, index: usize) -> Option<GridCoords> {
        self.path.get(index - 1).copied()
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