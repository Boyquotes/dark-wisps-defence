use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use crate::common_components::Health;
use crate::grids::common::GridCoords;
use crate::wisps::components::{Target, Wisp};

#[derive(Bundle, Default)]
struct WispBundle {
    wisp: Wisp,
    target: Target,
    grid_coords: GridCoords,
    health: Health,
    display: MaterialMesh2dBundle<ColorMaterial>,
}

pub fn spawn_wisp(
    commands: &mut Commands,
    meshes:  &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid_coords: GridCoords,
) -> Entity {
    commands.spawn(
        WispBundle {
            grid_coords,
            health: Health(100),
            display: MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(6.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(grid_coords.to_world_coords_centered().extend(0.)),
                ..default()
            },
            ..Default::default()
        }
    ).id()
}