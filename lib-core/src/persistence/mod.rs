use bevy::app::{App, Plugin};

pub mod common;
pub mod load;
pub mod save;

pub use rusqlite;

pub struct LoadSavePlugin;
impl Plugin for LoadSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            load::MapLoadPlugin,
            save::MapSavePlugin,
        ));
    }
}
pub mod load_save_prelude {
    pub use super::common::*;
    pub use super::load::*;
    pub use super::save::*;
    pub use super::rusqlite;
}
