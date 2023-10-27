use std::collections::BinaryHeap;
use bevy::prelude::*;
use crate::grids::common::GridCoords;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::grids::visited::{TrackingGrid};
use crate::search::common::{State, ALL_DIRECTIONS};


pub fn path_find_energy_beckon(grid: &Res<ObstacleGrid>, start_coords: GridCoords) -> Option<Vec<GridCoords>> {
    // BFS to find closest building field
    // TODO: TrackingGrids could probably be thread-local
    let mut tracking = TrackingGrid::new_with_size(grid.width, grid.height);
    let mut queue = BinaryHeap::new();
    queue.push(State{ cost: 0, coords: start_coords });
    tracking.set_tracked(start_coords, start_coords);
    while let Some(State{ cost, coords }) = queue.pop() {
        for (delta_x, delta_y) in ALL_DIRECTIONS {
            let new_coords = coords.shifted((delta_x, delta_y));
            if !new_coords.is_in_bounds(grid.bounds())
                || tracking.is_tracked(new_coords)
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

            tracking.set_tracked(new_coords, coords);
            let new_cost = match grid[new_coords] {
                Field::Building(_, building_type) => {
                    if building_type.is_energy_rich() {
                        // Compile the path by backtracking
                        return Some(tracking.compile_path(new_coords, start_coords));
                    } else {
                        cost + 10
                    }
                }
                _ => {
                    cost + 1
                },
            };
            queue.push(State { cost: new_cost, coords: new_coords });
        }
    }
    None
}