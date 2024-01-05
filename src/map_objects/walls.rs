use bevy::prelude::*;
use crate::common::Z_OBSTACLE;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::emissions::EmissionsEnergyRecalculateAll;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub const WALL_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

#[derive(Component)]
pub struct Wall;

#[derive(Bundle)]
pub struct BundleWall {
    sprite: SpriteBundle,
    grid_position: GridCoords,
    wall: Wall,
}

impl BundleWall {
    pub fn new(grid_position: GridCoords) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::GRAY, // Color::hsla(0., 0.5, 1.3, 0.8); for hdr
                    custom_size: Some(WALL_GRID_IMPRINT.world_size()),
                    ..Default::default()
                },
                transform: Transform::from_translation(
                    grid_position.to_world_position_centered(WALL_GRID_IMPRINT).extend(Z_OBSTACLE)
                ),
                ..Default::default()
            },
            grid_position,
            wall: Wall,
        }
    }
    pub fn spawn(
        self,
        commands: &mut Commands,
        emissions_energy_recalculate_all: &mut EmissionsEnergyRecalculateAll,
        obstacles_grid: &mut ObstacleGrid
    ) -> Entity {
        let position = self.grid_position;
        let entity = commands.spawn(self).id();
        obstacles_grid.imprint(position, Field::Wall(entity), WALL_GRID_IMPRINT);
        emissions_energy_recalculate_all.0 = true;
        entity
    }
}

pub fn remove_wall(
    commands: &mut Commands,
    emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) {
    let entity = match &obstacle_grid[grid_position] {
        Field::Wall(entity) => *entity,
        _ => panic!("Cannot remove a wall on a non-wall"),
    };
    commands.entity(entity).despawn();
    obstacle_grid.deprint(grid_position, WALL_GRID_IMPRINT);
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
    if !matches!(*grid_object_placer.single(), GridObjectPlacer::Wall) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a wall
        if obstacle_grid[mouse_coords].is_empty() {
            BundleWall::new(mouse_coords).spawn(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a wall
        if obstacle_grid[mouse_coords].is_wall() {
            remove_wall(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid, mouse_coords);
        }
    }
}

// TODO: decide whether to keep it
pub fn color_rotation_system(
    mut query: Query<&mut Sprite, With<Wall>>,
    time: Res<Time>,
) {
    for mut sprite in query.iter_mut() {
        if let Color::Hsla{hue, ..} = &mut sprite.color {
            *hue += time.delta_seconds() * 100.;
            if *hue > 360. {
                *hue = 0.;
            }
        }
    }
}