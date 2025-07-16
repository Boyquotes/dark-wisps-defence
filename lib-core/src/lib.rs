pub mod buildings;


pub mod prelude {
    pub use crate::buildings::prelude::*;
}

pub mod lib_prelude {
    pub use serde::{Deserialize, Serialize};
    pub use bevy::prelude::*;
}