use std::collections::VecDeque;
use bevy::prelude::*;
use crate::grids::common::GridCoords;
use crate::grids::emissions::{Emissions, EmissionsGrid, EmissionsType};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::grids::visited::VisitedGrid;
use crate::search::common::CARDINAL_DIRECTIONS;

/// Defines how to calculate the emissions as a function of distance
/// `Constant` - value is constant regardless of distance
/// `Linear` - value decreasing linearly with distance
#[derive(Clone)]
pub enum FloodEmissionsEvaluator {
    Constant(f32),
    Linear{growth: f32},
    ExponentialDecay{start_value: f32, decay: f32},
}

/// Describes what type of emissions, and how far to spread it.
/// The evaluator determines how to calculate the emissions value as the flood spreads
#[derive(Clone)]
pub struct FloodEmissionsDetails {
    pub emissions_type: EmissionsType,
    pub range: usize,
    pub evaluator: FloodEmissionsEvaluator,
}

pub fn flood_emissions(
    emissions_grid: &mut EmissionsGrid,
    obstacles_grid: &ObstacleGrid,
    start_coords: &Vec<GridCoords>,
    emissions_details: &Vec<FloodEmissionsDetails>,
    ignore_walls: bool,
    ignore_buildings: bool
) {
    let mut visited_grid = VisitedGrid::new_with_size(obstacles_grid.width, obstacles_grid.height);
    let mut queue = VecDeque::new();
    let max_range = emissions_details.iter().map(|details| details.range).max().unwrap();
    start_coords.iter().for_each(|coords| {
        queue.push_back((1, *coords));
        visited_grid.set_visited(*coords);
        for details in emissions_details {
            apply_emissions_details(emissions_grid, *coords, details, 1);
        }
    });
    while let Some((distance, coords)) = queue.pop_front() {
        for (delta_x, delta_y) in CARDINAL_DIRECTIONS {
            let new_coords = coords.shifted((delta_x, delta_y));
            if !new_coords.is_in_bounds(obstacles_grid.bounds())
                || visited_grid.is_visited(new_coords)
                || (!ignore_walls && obstacles_grid[new_coords].is_wall())
                || (!ignore_buildings && obstacles_grid[new_coords].is_building())
            {
                continue;
            }

            visited_grid.set_visited(new_coords);
            let new_distance = distance + 1;
            for details in emissions_details {
                if new_distance <= details.range {
                    apply_emissions_details(emissions_grid, new_coords, details, new_distance);
                }
            }
            if new_distance < max_range {
                queue.push_back((new_distance, new_coords));
            }
        }
    }
}

fn apply_emissions_details(
    emissions_grid: &mut EmissionsGrid,
    grid_coords: GridCoords,
    details: &FloodEmissionsDetails,
    distance: usize
) {
    let value = match details.evaluator {
        FloodEmissionsEvaluator::Constant(value) => value,
        FloodEmissionsEvaluator::Linear{growth} => {
            growth * distance as f32
        },
        FloodEmissionsEvaluator::ExponentialDecay{start_value, decay} => {
            start_value * (-1. * decay * distance as f32).exp()
        },
    };
    match details.emissions_type {
        EmissionsType::Energy => {
            emissions_grid.add_energy(grid_coords, value);
        }
    }
}

pub enum FloodEnergySupplyMode {
    Increase,
    Decrease,
}

pub fn flood_energy_supply(
    energy_supply_grid: &mut EnergySupplyGrid,
    start_coords: &Vec<GridCoords>,
    mode: FloodEnergySupplyMode,
    range: usize,
) {
    let mut visited_grid = VisitedGrid::new_with_size(energy_supply_grid.width, energy_supply_grid.height);
    let mut queue = VecDeque::new();
    start_coords.iter().for_each(|coords| {
        energy_supply_grid[*coords].increase_supply();
        queue.push_back((0, *coords));
        visited_grid.set_visited(*coords);
    });
    while let Some((distance, coords)) = queue.pop_front() {
        for (delta_x, delta_y) in CARDINAL_DIRECTIONS {
            let new_coords = coords.shifted((delta_x, delta_y));
            if !new_coords.is_in_bounds(energy_supply_grid.bounds())
                || visited_grid.is_visited(new_coords)
            {
                continue;
            }

            visited_grid.set_visited(new_coords);
            match mode {
                FloodEnergySupplyMode::Increase => energy_supply_grid[coords].increase_supply(),
                FloodEnergySupplyMode::Decrease => energy_supply_grid[coords].decrease_supply(),
            }
            let new_distance = distance + 1;
            if new_distance < range {
                queue.push_back((new_distance, new_coords));
            }
        }
    }
}