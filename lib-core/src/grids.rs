use std::{borrow::Borrow, collections::VecDeque, fmt::Debug};

use crate::lib_prelude::*;

pub mod grids_prelude {
    pub use super::*;
}

pub const CELL_SIZE: f32 = 16.;

pub type GridVersion = u32;

pub trait FieldTrait: Default + Clone + Debug {}
impl <T: Default + Clone + Debug> FieldTrait for T {}

pub trait GridVersionTrait: Default {}
impl <T: Default> GridVersionTrait for T {}

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
    pub fn is_imprint_in_bounds(&self, imprint: impl Borrow<GridImprint>, (width, height): (i32, i32)) -> bool {
        let imprint_world_size = imprint.borrow().bounds();
        self.x >= 0 && self.x + imprint_world_size.0 as i32 <= width && self.y >= 0 && self.y + imprint_world_size.1 as i32 <= height
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
    pub fn manhattan_distance(&self, other: &Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
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


#[derive(Component, Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
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
    pub fn bounds(&self) -> (i32, i32) {
        match self {
            GridImprint::Rectangle{width, height} => {
                (*width, *height)
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

#[derive(Component, Default)]
pub struct GridPath {
    pub grid_version: GridVersion,
    pub path: VecDeque<GridCoords>,
}
impl GridPath {
    pub fn next_in_path(&self) -> Option<GridCoords> {
        self.path.front().copied()
    }
    pub fn remove_first(&mut self) {
        self.path.pop_front();
    }
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
    pub fn distance(&self) -> usize {
        self.path.len()
    }
    pub fn at_distance(&self, index: usize) -> Option<GridCoords> {
        self.path.get(index - 1).copied()
    }
}