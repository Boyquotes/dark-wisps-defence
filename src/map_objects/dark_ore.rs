use bevy::prelude::*;
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::grids::emissions::EmissionsEnergyRecalculateAll;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub const DARK_ORE_GRID_WIDTH: i32 = 3;
pub const DARK_ORE_GRID_HEIGHT: i32 = 3;
pub const DARK_ORE_WORLD_WIDTH: f32 = CELL_SIZE * DARK_ORE_GRID_WIDTH as f32;
pub const DARK_ORE_WORLD_HEIGHT: f32 = CELL_SIZE * DARK_ORE_GRID_HEIGHT as f32;
pub const DARK_ORE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: DARK_ORE_GRID_WIDTH, height: DARK_ORE_GRID_HEIGHT };

#[derive(Component)]
pub struct DarkOre {
    amount: isize,
    grid_imprint: GridImprint,
}

pub fn create_dark_ore(
    commands: &mut Commands,
    emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    obstacles_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) -> Entity {
    if !obstacles_grid[grid_position].is_empty() {
        panic!("Cannot place dark ore on a non-empty field");
    }
    
    let world_position = grid_position.to_world_position();
    let world_position_centered = world_position + Vec2::new(DARK_ORE_WORLD_WIDTH, DARK_ORE_WORLD_HEIGHT) / 2.;

    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::INDIGO,
                custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                ..Default::default()
            },
            transform: Transform::from_translation(world_position_centered.extend(0.)),
            ..Default::default()
        },
        grid_position,
        DarkOre { amount: 10000, grid_imprint: DARK_ORE_GRID_IMPRINT },
    )).id();

    obstacles_grid.imprint(grid_position, Field::DarkOre(entity), DARK_ORE_GRID_IMPRINT);
    emissions_energy_recalculate_all.0 = true;
    entity
}

pub fn remove_dark_ore(
    commands: &mut Commands,
    emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) {
    let entity = match &obstacle_grid[grid_position] {
        Field::DarkOre(entity) => *entity,
        _ => panic!("Cannot remove a dark_ore on a non-dark_ore"),
    };
    commands.entity(entity).despawn();
    obstacle_grid.remove(grid_position, DARK_ORE_GRID_IMPRINT);
    emissions_energy_recalculate_all.0 = true;
}


pub fn onclick_spawn_system(
    mut commands: Commands,
    mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
) {
    if !matches!(*grid_object_placer.single(), GridObjectPlacer::DarkOre) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a dark_ore
        if obstacle_grid[mouse_coords].is_empty() {
            create_dark_ore(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid, mouse_coords);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a dark_ore
        if obstacle_grid[mouse_coords].is_dark_ore() {
            remove_dark_ore(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid, mouse_coords);
        }
    }
}