use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use crate::prelude::*;

pub type GridVersion = u32;

pub trait FieldTrait: Default + Clone + Debug {}
impl <T: Default + Clone + Debug> FieldTrait for T {}

pub trait GridVersionTrait: Default {}
impl <T: Default> GridVersionTrait for T {}

#[derive(Resource)]
pub struct BaseGrid<FieldType, GridVersionType> where FieldType: FieldTrait, GridVersionType: GridVersionTrait {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<FieldType>,
    pub version: GridVersionType, // Used to determine whether the grid has changed
}

impl<FieldType, GridVersionType> BaseGrid<FieldType, GridVersionType> where FieldType: FieldTrait, GridVersionType: GridVersionTrait {
    pub fn new_empty() -> Self {
        Self {
            width: 0,
            height: 0,
            grid: vec![],
            version: GridVersionType::default(),
        }
    }
    pub fn new_with_size(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            grid: vec![Default::default(); (width * height) as usize],
            version: GridVersionType::default(),
        }
    }
    pub fn resize_and_reset(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        // TODO: reuse existing grid
        self.grid = vec![Default::default(); (width * height) as usize];
    }
    pub fn index(&self, coords: GridCoords) -> usize {
        (coords.y * self.width + coords.x) as usize
    }
    pub fn bounds(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}


impl<FieldType, GridVersionType> Index<GridCoords> for BaseGrid<FieldType, GridVersionType> where FieldType: FieldTrait, GridVersionType: GridVersionTrait {
    type Output = FieldType;

    fn index(&self, coords: GridCoords) -> &Self::Output {
        if !coords.is_in_bounds(self.bounds()) {
            panic!("Index out of bounds");
        }
        let index = self.index(coords);
        &self.grid[index]
    }
}
impl<FieldType, GridVersionType>  IndexMut<GridCoords> for BaseGrid<FieldType, GridVersionType> where FieldType: FieldTrait, GridVersionType: GridVersionTrait {
    fn index_mut(&mut self, coords: GridCoords) -> &mut Self::Output {
        if !coords.is_in_bounds(self.bounds()) {
            panic!("Index out of bounds");
        }
        let index = self.index(coords);
        &mut self.grid[index]
    }
}

