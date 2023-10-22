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