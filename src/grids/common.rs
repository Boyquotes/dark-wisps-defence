use bevy::prelude::*;

pub const CELL_SIZE: f32 = 16.;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GridType {
    Obstacles,
}