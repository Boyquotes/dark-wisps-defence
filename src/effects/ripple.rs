use bevy::{
    color::palettes::css::RED, 
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin}
};

use crate::{grids::wisps::WispsGrid, prelude::*, wisps::components::Wisp};

pub struct RipplePlugin;
impl Plugin for RipplePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderRipple>()
            .add_plugins((
                Material2dPlugin::<RippleMaterial>::default(),
            ))
            .add_systems(Update, (
                ripple_propagate_system,
                ripple_hit_system,
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
                Mesh2d(meshes.add(Circle::new(radius))),
                MeshMaterial2d(ripple_materials.add(RippleMaterial {
                    current_radius: 0.0,
                    wave_width: 0.35,
                    wave_exponent: 0.8,
                })),
                Transform::from_translation(world_position.extend(Z_GROUND_EFFECT)),
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
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
pub struct Ripple {
    max_radius: f32, // Must be half the mesh size
    current_radius: f32,
}

fn ripple_propagate_system(
    mut commands: Commands,
    time: Res<Time>,
    mut wave_materials: ResMut<Assets<RippleMaterial>>,
    mut ripples: Query<(Entity, &mut Ripple, &Speed, &MeshMaterial2d<RippleMaterial>)>,
) {
    for (entity, mut ripple, speed, material_handle) in ripples.iter_mut() {
        let Some(material) = wave_materials.get_mut(material_handle) else { continue; };
        ripple.current_radius += speed.0 * time.delta_secs();
        if ripple.current_radius > ripple.max_radius {
            commands.entity(entity).despawn();
        }

        // Update the ripple radius. Shader needs a normalized value from 0.0 to 1.0 over the mesh size.
        let mesh_size = ripple.max_radius * 2.;
        material.current_radius = ripple.current_radius / mesh_size;
    }
}

pub fn ripple_hit_system(
    wisps_grid: Res<WispsGrid>,
    ripples: Query<(&Ripple, &Transform)>,
    mut wisps: Query<(&mut Health, &Transform), With<Wisp>>,
) {
    for (ripple, ripple_transform) in ripples.iter() {
        // Check all fields covered by the ripple for wisp collisions
        let starting_grid_coords = GridCoords::from_transform(&ripple_transform);
        let bounds_range = (ripple.current_radius / CELL_SIZE) as i32;
        // Make bounds +/-1 since the ripple starts from in-between the grid fields
        let lower_bound_x = std::cmp::max(0, starting_grid_coords.x - bounds_range - 1);
        let lower_bound_y = std::cmp::max(0, starting_grid_coords.y - bounds_range - 1);
        let upper_bound_x = std::cmp::min(wisps_grid.width - 1, starting_grid_coords.x + bounds_range + 1);
        let upper_bound_y = std::cmp::min(wisps_grid.height - 1, starting_grid_coords.y + bounds_range + 1);
        for x in lower_bound_x..=upper_bound_x {
            for y in lower_bound_y..=upper_bound_y {
                for wisp in &wisps_grid[GridCoords{ x, y }] {
                    let Ok((mut wisp_health, wisp_transform)) = wisps.get_mut(**wisp) else { continue; };
                    let distance = wisp_transform.translation.distance(ripple_transform.translation);
                    // Hit only wisps that are up to 5 units away from the front of the ripple
                    if distance > ripple.current_radius || distance < ripple.current_radius - 1. { continue; }
                    wisp_health.decrease(1);
                }
            }
        }
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