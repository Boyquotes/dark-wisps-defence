pub mod grids;
pub mod search;

pub mod lib_prelude {
    pub use serde::{Deserialize, Serialize};
    pub use bevy::prelude::*;
    pub use bevy::platform::collections::HashSet;

    pub use lib_core::prelude::*;
}

pub mod prelude {
    pub use crate::grids::energy_supply::{NeedsPower, HasPower, NoPower};
}