use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const CELL_SIZE: f32 = 16.;

// TODO: remove alongside generic targets from common.rs
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GridType {
    Obstacles,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Component, Hash)]
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
    pub fn to_world_position(&self) -> Vec2 {
        Vec2::new(self.x as f32 * CELL_SIZE, self.y as f32 * CELL_SIZE)
    }
    pub fn to_world_position_centered(&self, imprint: GridImprint) -> Vec2 {
        self.to_world_position() + imprint.world_center()
    }
    pub fn shifted(&self, (dx, dy): (i32, i32)) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
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
impl GridImprint {
    pub fn covered_coords(&self, coords: GridCoords) -> Vec<GridCoords> {
        match self {
            GridImprint::Rectangle{width, height} => {
                 (0..*height)
                     .flat_map(|y| (0..*width).map(move |x| coords.shifted((x, y))))
                     .collect()
            }
        }
    }

    pub fn world_size(&self) -> Vec2 {
        match self {
            GridImprint::Rectangle{width, height} => {
                Vec2::new(*width as f32 * CELL_SIZE, *height as f32 * CELL_SIZE)
            }
        }
    }
    pub fn world_center(&self) -> Vec2 {
        self.world_size() / 2.
    }
}

impl Default for GridImprint {
    fn default() -> Self {
        GridImprint::Rectangle{width: 1, height: 1}
    }
}