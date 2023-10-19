use std::collections::BinaryHeap;
use bevy::prelude::*;
use crate::grids::common::GridCoords;
use crate::grids::obstacles::{Field, ObstacleGrid};


pub fn path_find_closest_building(grid: &Res<ObstacleGrid>, start_coords: GridCoords) -> Option<Vec<GridCoords>> {
    // BFS to find closest building field
    let mut tracking = TrackingGrid::new(grid.width, grid.height);
    let mut queue = BinaryHeap::new();
    queue.push(State{ cost: 0, coords: start_coords });
    tracking.set_visited(start_coords, start_coords);
    while let Some(State{ cost, coords }) = queue.pop() {
        for &(delta_x, delta_y) in &[
            (0, 1), (1, 0), (0, -1), (-1, 0), // cardinal directions
            (1, 1), (-1, 1), (1, -1), (-1, -1), // diagonal directions
        ] {
            let new_coords = GridCoords{ x: coords.x + delta_x, y: coords.y + delta_y };
            if !new_coords.is_in_bounds(grid.bounds())
                || tracking.is_visited(new_coords)
                || grid[new_coords].is_wall()
            {
                continue;
            }

            // If it is a diagonal move it shall be allowed only if both adjacent fields are empty
            if delta_x.abs() == delta_y.abs() {
                let adjacent_x = (coords.x + delta_x, coords.y).into();
                let adjacent_y = (coords.x, coords.y + delta_y).into();
                if grid[adjacent_x].is_wall() || grid[adjacent_y].is_wall() {
                    continue;
                }
            }
            tracking.set_visited(new_coords, coords);
            match grid[new_coords] {
                Field::Building(_) => {
                    // Compile the path by backtracking
                    return Some(tracking.compile_path(new_coords, start_coords));
                }
                _ => {},
            }
            let new_cost = cost + 1;
            queue.push(State { cost: new_cost, coords: new_coords });
        }
    }

    None
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct State {
    pub cost: usize,
    pub coords: GridCoords,
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Visited {
    Unvisited,
    Visited{from: GridCoords},
}

struct TrackingGrid {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<Visited>,
}
impl TrackingGrid {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            grid: vec![Visited::Unvisited; (width * height) as usize],
        }
    }
    pub fn set_visited(&mut self, coords: GridCoords, from: GridCoords) {
        self.grid[(coords.y * self.width + coords.x) as usize] = Visited::Visited{from};
    }
    pub fn is_visited(&self, coords: GridCoords) -> bool {
        self.grid[(coords.y * self.width + coords.x) as usize] != Visited::Unvisited
    }
    // Creates a path between given points.
    // The path is created in reverse(backtracking "from" into "to") as during BFS we are saving from which field we came.
    pub fn compile_path(&self, from_coords: GridCoords, to_coords: GridCoords) -> Vec<GridCoords> {
        let mut path = vec![from_coords];
        let mut curr_coords = from_coords;
        loop {
            curr_coords = match self.grid[(curr_coords.y * self.width + curr_coords.x) as usize] {
                Visited::Unvisited => panic!("Encountered unvisited field during compilation of path"),
                Visited::Visited{from} => from,
            };
            if curr_coords != to_coords {
                path.push(curr_coords);
            } else { break; }
        }
        path.reverse();
        path
    }
}