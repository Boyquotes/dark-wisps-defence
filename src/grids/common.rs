use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const CELL_SIZE: f32 = 16.;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GridType {
    Obstacles,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Component)]
pub struct GridCoords {
    pub x: i32,
    pub y: i32,
}

impl GridCoords {
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            x: (transform.translation.x / CELL_SIZE).floor() as i32,
            y: (transform.translation.y / CELL_SIZE).floor() as i32,
        }
    }
    pub fn from_world_vec2(world_coords: Vec2) -> Self {
        Self {
            x: (world_coords.x / CELL_SIZE).floor() as i32,
            y: (world_coords.y / CELL_SIZE).floor() as i32,
        }
    }
    pub fn is_in_bounds(&self, (width, height): (i32, i32)) -> bool {
        self.x >= 0 && self.x < width && self.y >= 0 && self.y < height
    }
    pub fn to_world_coords(&self) -> Vec2 {
        Vec2::new(self.x as f32 * CELL_SIZE, self.y as f32 * CELL_SIZE)
    }
    pub fn to_world_coords_centered(&self) -> Vec2 {
        Vec2::new(self.x as f32 * CELL_SIZE + CELL_SIZE / 2., self.y as f32 * CELL_SIZE + CELL_SIZE / 2.)
    }
}

impl From<(i32, i32)> for GridCoords {
    fn from(coords: (i32, i32)) -> Self {
        Self {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl Into<(i32, i32)> for GridCoords {
    fn into(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridImprint {
    Rectangle{width: i32, height: i32},
}
