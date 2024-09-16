use bevy::{
    color::palettes::css::RED, 
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle,}
};

use crate::prelude::*;

pub struct RipplePlugin;
impl Plugin for RipplePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderRipple>()
            .add_plugins((
                Material2dPlugin::<RippleMaterial>::default(),
            ))
            .add_systems(Startup, (
                setup,
            ))
            .add_systems(Update, (
                update_ripple_system,
                spawn_random_wisps_effect_system
            ))
            .add_systems(PostUpdate, (
                BuilderRipple::spawn_system,
            ));
    }
}

#[derive(Event)]
pub struct BuilderRipple {
    pub world_position: Vec2,
    pub radius: f32, // in world size
}
impl BuilderRipple {
    pub fn new(world_position: Vec2, radius: f32) -> Self {
        Self { world_position, radius }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderRipple>,
        mut ripple_materials: ResMut<Assets<RippleMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for &BuilderRipple { world_position, radius } in events.read() {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::new(radius)).into(),
                    material: ripple_materials.add(RippleMaterial {
                        current_radius: 0.0,
                        wave_width: 0.35,
                        wave_exponent: 0.8,
                    }),
                    transform: Transform::from_translation(world_position.extend(Z_GROUND_EFFECT)),
                    ..default()
                },
                Ripple{ max_radius: radius, current_radius: 0. },
                Speed(70.0),
            ));
        }
    }

}
impl Command for BuilderRipple {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}


#[derive(Asset, TypePath, Debug, Clone, AsBindGroup)]
pub struct RippleMaterial {
    #[uniform(0)]
    pub current_radius: f32,
    #[uniform(0)]
    pub wave_width: f32,
    #[uniform(0)]
    pub wave_exponent: f32,
}

impl Material2d for RippleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ripple.wgsl".into()
    }
}

#[derive(Component)]
pub struct Ripple {
    max_radius: f32, // Must be half the mesh size
    current_radius: f32,
}

fn setup(
    mut commands: Commands,
    mut ripple_materials: ResMut<Assets<RippleMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let ripple_material_handle = ripple_materials.add(RippleMaterial {
        current_radius: 0.0,
        wave_width: 0.35,
        wave_exponent: 0.8,
    });

    let test_mesh = meshes.add(Circle::new(16. * 4.));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: test_mesh.into(),
            material: ripple_material_handle,
            transform: Transform {
                translation: Vec3::new(80.0, 10.0, 1.0),
                ..default()
            },
            ..default()
        },
        Ripple{ max_radius: 16. * 4., current_radius: 0. },
        Speed(70.0),
    ));
}

fn update_ripple_system(
    mut commands: Commands,
    time: Res<Time>,
    mut wave_materials: ResMut<Assets<RippleMaterial>>,
    mut ripples: Query<(Entity, &mut Ripple, &Speed, &Handle<RippleMaterial>)>,
) {
    for (entity, mut ripple, speed, material_handle) in ripples.iter_mut() {
        let Some(material) = wave_materials.get_mut(material_handle) else { continue; };
        ripple.current_radius += speed.0 * time.delta_seconds();
        if ripple.current_radius > ripple.max_radius {
            commands.entity(entity).despawn();
        }

        // Update the ripple radius. Shader needs a normalized value from 0.0 to 1.0 over the mesh size.
        let mesh_size = ripple.max_radius * 2.;
        material.current_radius = ripple.current_radius / mesh_size;
    }
}

fn spawn_random_wisps_effect_system(
    mut commands: Commands,
    button_input: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<crate::mouse::MouseInfo>,
) {
    if button_input.just_released(MouseButton::Left){
        println!("Spawning");
        commands.add(BuilderRipple::new(mouse_info.world_position, 16. * 4. * 4.));
    }
}

#[allow(dead_code)]
fn debug_draw_ripple_outline_system(
    mut gizmos: Gizmos,
    ripples: Query<(&Ripple, &Transform)>,
) {
    for (ripple, transform) in ripples.iter() {
        gizmos.circle_2d(transform.translation.xy(), ripple.current_radius, RED);
    }
}