use bevy::prelude::*;
use crate::grids::common::GridType;
use crate::grids::obstacles::{GridCoords};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GridDynamicObject {
    Wisp(Entity),
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum TargetType {
    #[default]
    None,
    Field{coords: GridCoords, grid_version: u32},
    DynamicObject(GridDynamicObject),
    Unreachable{grid_type: GridType, grid_version: u32},
}

impl TargetType {
    pub fn is_some(&self) -> bool {
        !matches!(self, TargetType::None) && !matches!(self, TargetType::Unreachable{..})
    }
    pub fn is_unreachable(&self) -> bool {
        matches!(self, TargetType::Unreachable{..})
    }
}