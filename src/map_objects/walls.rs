use bevy::prelude::*;
use crate::grids::common::{CELL_SIZE, GridCoords};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

#[derive(Component)]
pub struct Wall;

pub fn create_wall(commands: &mut Commands, grid: &mut ResMut<ObstacleGrid>, grid_position: GridCoords) -> Entity {
    match grid[grid_position] {
        Field::Empty { .. } => {}
        _ => panic!("Cannot place a wall on a non-empty field"),
    }

    let world_position = Vec3::new(grid_position.x as f32 * CELL_SIZE, grid_position.y as f32 * CELL_SIZE, 0.);
    let mut color = Color::hsla(300., 0.5, 10.5, 0.1);
    let color = Color::rgba(0.6, 2.0, 2.2, 0.8);

    let entity = commands.spawn(
        SpriteBundle {
            sprite: Sprite {
                color,
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


pub fn onclick_wall_spawn_system(
    mut commands: Commands,
    mut grid: ResMut<ObstacleGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
) {
    if !matches!(*grid_object_placer.single(), GridObjectPlacer::Wall) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if !mouse_coords.is_in_bounds(grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a wall
        if grid[mouse_coords].is_empty() {
            create_wall(&mut commands, &mut grid, mouse_coords);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a wall
        if grid[mouse_coords].is_wall() {
            remove_wall(&mut commands, &mut grid, mouse_coords);
        }
    }
}