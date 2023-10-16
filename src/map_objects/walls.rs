use bevy::prelude::*;
use crate::grid::{CELL_SIZE, Field, ObstacleGrid, GridCoords};

#[derive(Component)]
pub struct Wall;

pub fn create_wall(commands: &mut Commands, grid: &mut ResMut<ObstacleGrid>, grid_position: GridCoords) -> Entity {
    match grid[grid_position] {
        Field::Empty { .. } => {}
        _ => panic!("Cannot place a wall on a non-empty field"),
    }

    let world_position = Vec3::new(grid_position.x as f32 * CELL_SIZE, grid_position.y as f32 * CELL_SIZE, 0.);
    let entity = commands.spawn(
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                ..Default::default()
            },
            transform: Transform::from_translation(world_position + Vec3::new(CELL_SIZE/2., CELL_SIZE/2., 0.0)),
            ..Default::default()
        }
    ).insert(
        grid_position
    ).insert(
        Wall
    ).id();

    grid.imprint_wall(grid_position, entity);
    entity
}

pub fn remove_wall(commands: &mut Commands, grid: &mut ResMut<ObstacleGrid>, grid_position: GridCoords) {
    let entity = match &grid[grid_position] {
        Field::Wall(entity) => *entity,
        _ => panic!("Cannot remove a wall on a non-wall"),
    };
    commands.entity(entity).despawn();
    grid.remove_wall(grid_position);
}