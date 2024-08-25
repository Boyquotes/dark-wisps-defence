use bevy::color::palettes::css::PURPLE;
use crate::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use crate::common::Z_WISP;
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridImprint};
use crate::wisps::components::{Target, Wisp};

pub const WISP_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

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
            health: Health(10),
            display: MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(6.)).into(),
                material: materials.add(ColorMaterial::from_color(PURPLE)),
                transform: Transform::from_translation(
                    grid_coords.to_world_position_centered(WISP_GRID_IMPRINT).extend(Z_WISP)
                ),
                ..default()
            },
            ..Default::default()
        }
    ).id()
}