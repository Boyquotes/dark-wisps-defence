use std::collections::VecDeque;
use bevy::prelude::*;
use crate::grids::common::GridCoords;
use crate::grids::emissions::{Emissions, EmissionsGrid, EmissionsType};
use crate::grids::obstacles::ObstacleGrid;
use crate::grids::visited::VisitedGrid;
use crate::search::common::CARDINAL_DIRECTIONS;

/// Defines how to calculate the emissions as a function of distance
/// `Constant` - value is constant regardless of distance
/// `Linear` - value decreasing linearly with distance
pub enum FloodEmissionsEvaluator {
    Constant(f32),
    Linear{value: f32, scale: f32},
}

/// Describes what type of emissions, and how far to spread it.
/// The evaluator determines how to calculate the emissions value as the flood spreads
pub struct FloodEmissionsDetails {
    pub emissions_type: EmissionsType,
    pub range: usize,
    pub evaluator: FloodEmissionsEvaluator,
}

pub fn flood_emissions(
    mut emissions_grid: ResMut<EmissionsGrid>,
    obstacles_grid: Res<ObstacleGrid>,
    start_coords: Vec<GridCoords>,
    emissions_details: Vec<FloodEmissionsDetails>,
    ignore_obstacles: bool
) {
    let mut visited_grid = VisitedGrid::new_with_size(obstacles_grid.width, obstacles_grid.height);
    let mut queue = VecDeque::new();
    let max_range = emissions_details.iter().map(|details| details.range).max().unwrap();
    start_coords.into_iter().for_each(|coords| {
        queue.push_back((0, coords));
        visited_grid.set_visited(coords);
        let emissions = &mut emissions_grid[coords];
        for details in &emissions_details {
            apply_emissions_details(emissions, details, 0);
        }
    });
    while let Some((distance, coords)) = queue.pop_front() {
        for (delta_x, delta_y) in CARDINAL_DIRECTIONS {
            let new_coords = coords.shifted((delta_x, delta_y));
            if !new_coords.is_in_bounds(obstacles_grid.bounds())
                || visited_grid.is_visited(new_coords)
                || (!ignore_obstacles && obstacles_grid[new_coords].is_obstacle())
            {
                continue;
            }

            visited_grid.set_visited(new_coords);
            let new_distance = distance + 1;
            let emissions = &mut emissions_grid[coords];
            for details in &emissions_details {
                if new_distance <= details.range {
                    apply_emissions_details(emissions, details, new_distance);
                }
            }
            if new_distance < max_range {
                queue.push_back((new_distance, new_coords));
            }
        }
    }
}

fn apply_emissions_details(
    emissions: &mut Emissions,
    details: &FloodEmissionsDetails,
    distance: usize
) {
    let value = match details.evaluator {
        FloodEmissionsEvaluator::Constant(value) => value,
        FloodEmissionsEvaluator::Linear{value, scale} => {
            value - scale * distance as f32
        }
    };
    match details.emissions_type {
        EmissionsType::Energy => {
            emissions.energy += value;
        }
    }
}