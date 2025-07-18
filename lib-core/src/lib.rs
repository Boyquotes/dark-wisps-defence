use bevy::app::{App, Plugin};

pub mod buildings;
mod camera;
pub mod grids;
pub mod mouse;
pub mod states;

pub struct LibCorePlugin;
impl Plugin for LibCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::CameraPlugin, mouse::MousePlugin, states::StatesPlugin));
    }
}


pub mod prelude {
    pub use crate::buildings::buildings_prelude::*;
    pub use crate::grids::grids_prelude::*;
    pub use crate::mouse::mouse_prelude::*;
    pub use crate::states::states_prelude::*;
}

pub mod lib_prelude {
    pub use serde::{Deserialize, Serialize};
    pub use bevy::prelude::*;
}