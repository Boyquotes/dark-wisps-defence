use bevy::render::render_resource::{ShaderRef, AsBindGroup};
use bevy::sprite::{Material2d, MaterialMesh2dBundle};

use crate::prelude::*;

use super::components::{WispAttackRange, WispChargeAttack, WispState, Wisp};

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
        asset_server: Res<AssetServer>,
        mut events: EventReader<BuilderWisp>,
        mut meshes: ResMut<Assets<Mesh>>,
        //mut materials: ResMut<Assets<ColorMaterial>>,
        mut materials: ResMut<Assets<WispMaterial>>,
    ) {
        let mut rng = nanorand::tls_rng();
        for &BuilderWisp { entity, grid_coords } in events.read() {

            let wisp_world_size = WISP_GRID_IMPRINT.world_size();
            commands.entity(entity).insert((
                grid_coords,
                Health(10),
                Speed(30.),
                // OLD PURPLE CIRCLE MATERIAL
                // MaterialMesh2dBundle {
                //     mesh: meshes.add(Circle::new(6.)).into(),
                //     material: materials.add(ColorMaterial::from_color(PURPLE)),
                //     transform: Transform::from_translation(
                //         grid_coords.to_world_position_centered(WISP_GRID_IMPRINT).extend(Z_WISP)
                //     ),
                //     ..default()
                // },
                MaterialMesh2dBundle {
                    mesh:  meshes.add(Rectangle::new(wisp_world_size.x, wisp_world_size.y)).into(),
                    material: materials.add(WispMaterial {
                        amplitude: rng.generate::<f32>() * 0.2 + 0.25, // 0.25 - 0.45
                        frequency: rng.generate::<f32>() * 5. + 15., // 15 - 20
                        speed: rng.generate::<f32>() * 3. + 4., // 4 - 7
                        sinus_direction: [-1., 1.][rng.generate::<usize>() % 2],
                        cosinus_direction: [-1., 1.][rng.generate::<usize>() % 2],
                        wisp_tex1: asset_server.load("wisps/tri_wisp.png"),
                        wisp_tex2: asset_server.load("wisps/big_wisp.png"),
                    }),
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

#[derive(Asset, TypePath, Debug, Clone, AsBindGroup)]
pub struct WispMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub wisp_tex1: Handle<Image>,
    #[texture(2)]
    pub wisp_tex2: Handle<Image>,

    #[uniform(4)]
    pub amplitude: f32,
    #[uniform(4)]
    pub frequency: f32,
    #[uniform(4)]
    pub speed: f32,
    #[uniform(4)]
    pub sinus_direction: f32,
    #[uniform(4)]
    pub cosinus_direction: f32,
}

impl Material2d for WispMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wisp.wgsl".into()
    }
}