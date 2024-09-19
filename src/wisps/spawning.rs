use bevy::color::palettes::css::PURPLE;
use bevy::sprite::MaterialMesh2dBundle;
use crate::prelude::*;
use crate::wisps::components::Wisp;

use super::components::{WispAttackRange, WispChargeAttack, WispState};

pub const WISP_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

#[derive(Event)]
pub struct BuilderWisp {
    pub entity: Entity,
    pub grid_coords: GridCoords,
}

impl BuilderWisp {
    pub fn new(entity: Entity, grid_coords: GridCoords) -> Self {
        Self { entity, grid_coords }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderWisp>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        for &BuilderWisp { entity, grid_coords } in events.read() {
            commands.entity(entity).insert((
                grid_coords,
                Health(10),
                Speed(30.),
                MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::new(6.)).into(),
                    material: materials.add(ColorMaterial::from_color(PURPLE)),
                    transform: Transform::from_translation(
                        grid_coords.to_world_position_centered(WISP_GRID_IMPRINT).extend(Z_WISP)
                    ),
                    ..default()
                },
                Wisp,
                WispState::default(),
                WispChargeAttack::default(),
                WispAttackRange(1),
                GridPath::default(),
            ));
        }
    }
}
impl Command for BuilderWisp {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}