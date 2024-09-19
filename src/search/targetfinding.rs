use std::collections::BinaryHeap;
use crate::prelude::*;
use crate::grids::common::GridCoords;
use crate::grids::obstacles::ObstacleGrid;
use crate::grids::visited::VisitedGrid;
use crate::grids::wisps::WispsGrid;
use crate::search::common::{CARDINAL_DIRECTIONS, State};
use crate::wisps::components::WispEntity;

/// Finds the closest wisp
/// `ignore_obstacles` ignores all grid obstacles
/// `range` is the maximum searching range, diagonal moves are not allowed
/// Returns grid coords and entity id of the closest wisp or None if no wisp is found
pub fn target_find_closest_wisp(
    obstacle_grid: &Res<ObstacleGrid>,
    wisps_grid: &Res<WispsGrid>,
    start_coords: Vec<GridCoords>,
    range: usize,
    ignore_obstacles: bool,
) -> Option<(GridCoords, WispEntity)> {
    let mut visited_grid = VisitedGrid::new_with_size(obstacle_grid.width, obstacle_grid.height);
    let mut queue = BinaryHeap::new();
    start_coords.into_iter().for_each(
        |coords| {
            queue.push(State{cost: usize::MIN, distance: 0, coords });
            visited_grid.set_visited(coords);
        }
    );
    while let Some(State{ cost, distance, coords }) = queue.pop() {
        for (delta_x, delta_y) in CARDINAL_DIRECTIONS {
            let new_coords = coords.shifted((delta_x, delta_y));
            if distance > range
                || !new_coords.is_in_bounds(obstacle_grid.bounds())
                || visited_grid.is_visited(new_coords)
                || (!ignore_obstacles && !obstacle_grid[new_coords].is_empty())
            {
                continue;
            }

            if !wisps_grid[new_coords].is_empty() {
                return Some((new_coords, wisps_grid[new_coords][0]));
            }

            visited_grid.set_visited(new_coords);
            queue.push(State{ cost: cost + 1, distance: distance + 1, coords: new_coords });
        }
    }
    None
}