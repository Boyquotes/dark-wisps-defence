use bevy::color::palettes::css::GRAY;

use lib_grid::grids::emissions::EmissionsEnergyRecalculateAll;
use lib_grid::grids::obstacles::{Field, ObstacleGrid};

use crate::prelude::*;
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub struct WallPlugin;
impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderWall>()
            .add_systems(PostUpdate, (
                BuilderWall::spawn_system,
            ))
            .add_systems(Update, (
                onclick_spawn_system,
                color_rotation_system,
            ));
    }
}

pub const WALL_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

#[derive(Component)]
pub struct Wall;

#[derive(Event)]
pub struct BuilderWall {
    pub entity: Entity,
    pub grid_position: GridCoords,
}

impl BuilderWall {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self { 
        Self { entity, grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderWall>,
        mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
     ) {
        for &BuilderWall { entity, grid_position } in events.read() {
            commands.entity(entity).insert((
                Sprite {
                    color: GRAY.into(), // Color::hsla(0., 0.5, 1.3, 0.8); for hdr
                    custom_size: Some(WALL_GRID_IMPRINT.world_size()),
                    ..default()
                },
                Transform::from_translation(grid_position.to_world_position_centered(WALL_GRID_IMPRINT).extend(Z_OBSTACLE)),
                grid_position,
                WALL_GRID_IMPRINT,
                Wall,
            ));
            emissions_energy_recalculate_all.0 = true;
        }
    }
}
impl Command for BuilderWall {
    fn apply(self, world: &mut World) {
        world.send_event(self);
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
    obstacle_grid.deprint_all(grid_position, WALL_GRID_IMPRINT);
    emissions_energy_recalculate_all.0 = true;
}


pub fn onclick_spawn_system(
    mut commands: Commands,
    mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
) {
    if !matches!(*grid_object_placer.single().unwrap(), GridObjectPlacer::Wall) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a wall
        if obstacle_grid[mouse_coords].is_empty() {
            let wall_entity = commands.spawn_empty().id();
            commands.queue(BuilderWall::new(wall_entity, mouse_coords));
            obstacle_grid.imprint(mouse_coords, Field::Wall(wall_entity), WALL_GRID_IMPRINT);
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
        if let Color::Hsla(Hsla{hue, ..}) = &mut sprite.color {
            *hue += time.delta_secs() * 100.;
            if *hue > 360. {
                *hue = 0.;
            }
        }
    }
}