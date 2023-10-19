use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use bevy::prelude::*;
use crate::grids::common::GridCoords;


#[derive(Resource)]
pub struct GridBase<FieldType> where FieldType: Default + Clone + Debug {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<FieldType>,
    pub version: u32, // Used to determine whether the grid has changed
}

impl <FieldType>GridBase<FieldType> where FieldType: Default + Clone + Debug {
    pub fn new_empty() -> Self {
        Self {
            width: 0,
            height: 0,
            grid: vec![],
            version: 0,
        }
    }
    pub fn resize_and_reset(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.grid = vec![Default::default(); (width * height) as usize];
    }
    pub fn bounds(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}


impl<FieldType> Index<GridCoords> for GridBase<FieldType> where FieldType: Default + Clone + Debug {
    type Output = FieldType;

    fn index(&self, coords: GridCoords) -> &Self::Output {
        if !coords.is_in_bounds(self.bounds()) {
            panic!("Index out of bounds");
        }
        let index = (coords.y * self.width + coords.x) as usize;
        &self.grid[index]
    }
}
impl<FieldType>  IndexMut<GridCoords> for GridBase<FieldType> where FieldType: Default + Clone + Debug {
    fn index_mut(&mut self, coords: GridCoords) -> &mut Self::Output {
        if !coords.is_in_bounds(self.bounds()) {
            panic!("Index out of bounds");
        }
        let index = (coords.y * self.width + coords.x) as usize;
        &mut self.grid[index]
    }
}

