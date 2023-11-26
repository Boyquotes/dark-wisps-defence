use bevy::prelude::*;
use crate::common::Z_OBSTACLE;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::emissions::EmissionsEnergyRecalculateAll;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub const DARK_ORE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };

#[derive(Component)]
pub struct DarkOre {
    amount: isize,
    grid_imprint: GridImprint,
}

pub fn create_dark_ore(
    commands: &mut Commands,
    asset_server: &AssetServer,
    emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    obstacles_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) -> Entity {
    if !obstacles_grid[grid_position].is_empty() {
        panic!("Cannot place dark ore on a non-empty field");
    }

    let entity = commands.spawn((
        get_dark_ore_sprite_bundle(grid_position, asset_server),
        grid_position,
        DarkOre { amount: 10000, grid_imprint: DARK_ORE_GRID_IMPRINT },
    )).id();

    obstacles_grid.imprint(grid_position, Field::DarkOre(entity), DARK_ORE_GRID_IMPRINT);
    emissions_energy_recalculate_all.0 = true;
    entity
}

pub fn get_dark_ore_sprite_bundle(grid_position: GridCoords, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(DARK_ORE_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load("map_objects/dark_ore.png"),
        transform: Transform::from_translation(
            grid_position.to_world_position_centered(DARK_ORE_GRID_IMPRINT).extend(Z_OBSTACLE)
        ),
        ..Default::default()
    }
}

pub fn remove_dark_ore(
    commands: &mut Commands,
    emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) {
    let entity = match &obstacle_grid[grid_position] {
        Field::DarkOre(entity) => *entity,
        _ => panic!("Cannot remove a dark_ore from a non-dark_ore field"),
    };
    commands.entity(entity).despawn();
    obstacle_grid.deprint(grid_position, DARK_ORE_GRID_IMPRINT);
    emissions_energy_recalculate_all.0 = true;
}


pub fn onclick_spawn_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut obstacles_grid: ResMut<ObstacleGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
    dark_ores_query: Query<&GridCoords, With<DarkOre>>,
) {
    if !matches!(*grid_object_placer.single(), GridObjectPlacer::DarkOre) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if !mouse_coords.is_in_bounds(obstacles_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a dark_ore
        if obstacles_grid[mouse_coords].is_empty() {
            create_dark_ore(&mut commands, &asset_server, &mut emissions_energy_recalculate_all, &mut obstacles_grid, mouse_coords);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a dark_ore
        match obstacles_grid[mouse_coords] {
            Field::DarkOre(entity) => {
                if let Ok(dark_ore_coords) = dark_ores_query.get(entity) {
                    remove_dark_ore(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacles_grid, *dark_ore_coords);
                }
            },
            _ => {}
        }
    }
}