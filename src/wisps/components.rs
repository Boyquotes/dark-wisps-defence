use bevy::prelude::*;
use crate::common::TargetType;
use crate::grid::GridCoords;


#[derive(Component, Default)]
pub struct Wisp;

#[derive(Component, Default, Debug)]
pub struct Target{
    pub target_type: TargetType,
    pub grid_path: Option<Vec<GridCoords>>,
}

impl Target {
    pub fn is_on_its_path(&self) -> bool
    {
        self.target_type.is_some() && self.grid_path.is_some()
    }
    // We are at the destination if there is a target, and no path anymore.
    pub fn is_at_destination(&self) -> bool {
        self.target_type.is_some() && self.grid_path.is_none()
    }
    pub fn is_unreachable(&self) -> bool {
        self.target_type.is_unreachable()
    }
}