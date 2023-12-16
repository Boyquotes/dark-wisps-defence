use bevy::prelude::*;
use crate::common::Z_OBSTACLE;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

#[derive(Component, Clone, Debug)]
pub struct QuantumField {
    pub grid_imprint: GridImprint,
}

pub fn create_quantum_field(
    commands: &mut Commands,
    obstacles_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
    grid_imprint: GridImprint,
) -> Entity {
    let entity = commands.spawn((
        get_quantum_field_sprite_bundle(grid_position, grid_imprint),
        grid_position,
        QuantumField { grid_imprint },
    )).id();

    obstacles_grid.imprint(grid_position, Field::QuantumField(entity), grid_imprint);
    entity
}

pub fn get_quantum_field_sprite_bundle(grid_position: GridCoords, grid_imprint: GridImprint) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(grid_imprint.world_size()),
            color: Color::INDIGO,
            ..Default::default()
        },
        transform: Transform::from_translation(
            grid_position.to_world_position_centered(grid_imprint).extend(Z_OBSTACLE)
        ),
        ..Default::default()
    }
}

pub fn remove_quantum_field(
    commands: &mut Commands,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    entity: Entity,
    grid_position: GridCoords,
    grid_imprint: GridImprint,
) {
    commands.entity(entity).despawn();
    obstacle_grid.deprint(grid_position, grid_imprint);
}


pub fn onclick_spawn_system(
    mut commands: Commands,
    mut obstacles_grid: ResMut<ObstacleGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
    quantum_fields_query: Query<&GridCoords, With<QuantumField>>,
) {
    let GridObjectPlacer::QuantumField(quantum_field) = grid_object_placer.single() else { return; };
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacles_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a quantum_field
        if obstacles_grid[mouse_coords].is_empty() {
            create_quantum_field(&mut commands, &mut obstacles_grid, mouse_coords, quantum_field.grid_imprint);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a quantum_field
        match obstacles_grid[mouse_coords] {
            Field::QuantumField(entity) => {
                if let Ok(quantum_field_coords) = quantum_fields_query.get(entity) {
                    remove_quantum_field(&mut commands, &mut obstacles_grid, entity, *quantum_field_coords, quantum_field.grid_imprint);
                }
            },
            _ => {}
        }
    }
}