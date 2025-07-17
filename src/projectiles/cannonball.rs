use std::f32::consts::PI;

use lib_grid::search::common::ALL_DIRECTIONS;
use lib_grid::grids::wisps::WispsGrid;

use crate::prelude::*;
use crate::effects::explosions::BuilderExplosion;
use crate::projectiles::components::MarkerProjectile;
use crate::wisps::components::Wisp;


pub struct CannonballPlugin;
impl Plugin for CannonballPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderCannonball>()
            .add_systems(Update, (
                (
                    cannonball_move_system,
                    cannonball_hit_system,
                ).run_if(in_state(GameState::Running)),
            ))
            .add_systems(PostUpdate, (
                BuilderCannonball::spawn_system,
            ));
    }
}

pub const CANNONBALL_BASE_IMAGE: &str = "projectiles/cannonball.png";

#[derive(Component)]
pub struct MarkerCannonball;

// Cannonball follows Wisp, and if the wisp no longer exists, follows to the target position
#[derive(Component, Default)]
pub struct CannonballTarget{
    pub initial_distance: f32,
    pub target_position: Vec2,
}

#[derive(Event)]
pub struct BuilderCannonball {
    world_position: Vec2,
    target_position: Vec2,
}

impl BuilderCannonball {
    pub fn new(world_position: Vec2, target_position: Vec2) -> Self {
        Self {
            world_position,
            target_position,
        }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderCannonball>,
        asset_server: Res<AssetServer>,
    ) {
        for &BuilderCannonball{ world_position, target_position } in events.read() {
            commands.spawn((
                Sprite {
                    image: asset_server.load(CANNONBALL_BASE_IMAGE),
                    custom_size: Some(Vec2::new(8.0, 8.0)),
                    ..default()
                },
                Transform {
                    translation: world_position.extend(Z_PROJECTILE),
                    ..default()
                },
                MarkerProjectile,
                MarkerCannonball,
                CannonballTarget {
                    initial_distance: world_position.distance(target_position),
                    target_position,
                },
            ));
        }
    }
}
impl Command for BuilderCannonball {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn cannonball_move_system(
    mut cannonballs: Query<(&mut Transform, &CannonballTarget), With<MarkerCannonball>>,
    time: Res<Time>,
) {
    for (mut transform, target) in cannonballs.iter_mut() {
        let direction_vector = (target.target_position - transform.translation.xy()).normalize();
        let move_distance = direction_vector * time.delta_secs() * 200.;

        let remaining_distance = (transform.translation.xy() + move_distance).distance(target.target_position);

        // Calculate the progress as a value between 0 and 1
        let progress = 1. - remaining_distance / target.initial_distance;

        // Determine the scaling factor based on progress, applying a sine function for non-linearity
        let scale_factor = if progress <= 0.5 {
            1.0 + (PI * progress).sin()  // Non-linear scale up in the first half
        } else {
            (PI * (1.0 - progress)).sin() + 1.0  // Non-linear scale down in the second half
        };
        transform.scale = Vec3::splat(scale_factor);

        // Move the cannonball
        transform.translation += move_distance.extend(0.);
    }
}

pub fn cannonball_hit_system(
    mut commands: Commands,
    cannonballs: Query<(Entity, &Transform, &CannonballTarget), With<MarkerCannonball>>,
    wisps_grid: Res<WispsGrid>,
    mut wisps: Query<&mut Health, With<Wisp>>,
) {
    for (entity, cannonball_transform, target) in cannonballs.iter() {
        if cannonball_transform.translation.xy().distance(target.target_position) > 4. { continue; } // TODO: 1. and 2. are causing cannonballs jitters at landing. Investigate.

        let coords = GridCoords::from_transform(&cannonball_transform);
        for (dx, dy) in ALL_DIRECTIONS.iter().chain(&[(0, 0)]) {
            let blast_zone_coords = coords.shifted((*dx, *dy));
            if !blast_zone_coords.is_in_bounds(wisps_grid.bounds()) { continue; }

            commands.spawn(BuilderExplosion(blast_zone_coords));

            let wisps_in_coords = &wisps_grid[blast_zone_coords];
            for wisp in wisps_in_coords {
                let Ok(mut health) = wisps.get_mut(*wisp) else { continue }; // May not find wisp if the wisp spawned at the same frame.
                health.decrease(100);
            }
        }
        commands.entity(entity).despawn();
    }
}