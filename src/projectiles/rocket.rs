use bevy::sprite::Anchor;

use lib_grid::search::common::ALL_DIRECTIONS;
use lib_grid::grids::wisps::WispsGrid;

use crate::prelude::*;
use crate::projectiles::components::MarkerProjectile;
use crate::wisps::components::Wisp;
use crate::effects::explosions::BuilderExplosion;


pub struct RocketPlugin;
impl Plugin for RocketPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderRocket>()
            .add_systems(Update, (
                exhaust_blinking_system,
                (
                    rocket_move_system,
                    rocket_hit_system,
                ).run_if(in_state(GameState::Running)),
            ))
            .add_systems(PostUpdate, (
                BuilderRocket::spawn_system,
            ));
    }
}

pub const ROCKET_BASE_IMAGE: &str = "projectiles/rocket.png";
pub const ROCKET_EXHAUST_IMAGE: &str = "projectiles/rocket_exhaust.png";

#[derive(Component)]
pub struct MarkerRocket;
#[derive(Component)]
pub struct MarkerRocketExhaust;

// Rocket follows Wisp, and if the wisp no longer exists, looks for another target
#[derive(Component)]
pub struct RocketTarget(pub Entity);

#[derive(Event)]
pub struct BuilderRocket {
    world_position: Vec2,
    rotation: Quat,
    target_wisp: Entity,
}
impl BuilderRocket {
    pub fn new(world_position: Vec2, rotation: Quat, target_wisp: Entity) -> Self {
        Self {
            world_position,
            rotation,
            target_wisp,
        }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderRocket>,
        asset_server: Res<AssetServer>,
    ) {
        for &BuilderRocket{ world_position, rotation, target_wisp } in events.read() {
            let exhaust = commands.spawn((
                Sprite {
                    image: asset_server.load(ROCKET_EXHAUST_IMAGE),
                    custom_size: Some(Vec2::new(10.0, 6.25)),
                    anchor: Anchor::Custom(Vec2::new(0.75, 0.)),
                    ..default()
                },
                Transform {
                    translation: Vec2::ZERO.extend(Z_PROJECTILE_UNDER),
                    ..default()
                },
                MarkerRocketExhaust,
            )).id();
            commands.spawn((
                Sprite {
                    image: asset_server.load(ROCKET_BASE_IMAGE),
                    custom_size: Some(Vec2::new(20.0, 10.0)),
                    ..Default::default()
                },
                Transform {
                    translation: world_position.extend(Z_PROJECTILE),
                    rotation,
                    ..default()
                },
                MarkerProjectile,
                MarkerRocket,
                RocketTarget(target_wisp),
            )).add_child(exhaust);
        }
    }
}
impl Command for BuilderRocket {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn rocket_move_system(
    mut rockets: Query<(&mut Transform, &mut RocketTarget), With<MarkerRocket>>,
    time: Res<Time>,
    wisps: Query<(Entity, &Transform), (With<Wisp>, Without<MarkerRocket>)>,
) {
    let mut wisps_iter = wisps.iter();
    for (mut transform, mut target) in rockets.iter_mut() {
        let target_position = if let Ok((_, wisp_transform)) = wisps.get(target.0) {
            wisp_transform.translation.xy()
        } else {
            wisps_iter.next().map_or(Vec2::ZERO, |(wisp_entity, wisp_transform)| {
                target.0 = wisp_entity;
                wisp_transform.translation.xy()
            })
        };

        // Calculate the direction vector to the target
        let direction_vector = (target_position - transform.translation.xy()).normalize();

        // Calculate the current forward direction (assuming it's the local y-axis)
        let current_direction = transform.local_x().xy();

        // Move the entity forward (along the local y-axis)
        transform.translation += (current_direction * time.delta_secs() * 200.0).extend(0.0);

        // Calculate the target angle
        let target_angle = direction_vector.y.atan2(direction_vector.x);
        let current_angle = current_direction.y.atan2(current_direction.x);

        // Calculate the shortest rotation to the target angle
        let mut angle_diff = target_angle - current_angle;
        if angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        } else if angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }

        // Apply the rotation smoothly
        let rotation_speed = 1.5; // radians per second
        let max_rotation_speed = rotation_speed * time.delta_secs();
        let rotation_amount = angle_diff.clamp(-max_rotation_speed, max_rotation_speed);
        transform.rotate(Quat::from_rotation_z(rotation_amount));

    }
}

pub fn rocket_hit_system(
    mut commands: Commands,
    rockets: Query<(Entity, &Transform, &RocketTarget), (With<MarkerRocket>, Without<Wisp>)>,
    wisps_grid: Res<WispsGrid>,
    wisps_transforms: Query<&Transform, (With<Wisp>, Without<MarkerRocket>)>,
    mut wisps_health: Query<&mut Health, With<Wisp>>,
) {
    for (entity, rocket_transform, target) in rockets.iter() {
        let rocket_coords = GridCoords::from_transform(&rocket_transform);
        if !rocket_coords.is_in_bounds(wisps_grid.bounds()) {
            commands.entity(entity).despawn();
            continue;
        }

        let Ok(wisp_transform) = wisps_transforms.get(target.0) else { continue };
        if rocket_transform.translation.xy().distance(wisp_transform.translation.xy()) > 6. { continue; }

        let coords = GridCoords::from_transform(&rocket_transform);
        for (dx, dy) in ALL_DIRECTIONS.iter().chain(&[(0, 0)]) {
            let blast_zone_coords = coords.shifted((*dx, *dy));
            if !blast_zone_coords.is_in_bounds(wisps_grid.bounds()) { continue; }

            commands.spawn(BuilderExplosion(blast_zone_coords));

            let wisps_in_coords = &wisps_grid[blast_zone_coords];
            for wisp in wisps_in_coords {
                let Ok(mut health) = wisps_health.get_mut(*wisp) else { continue }; // May not find wisp if the wisp spawned at the same frame.
                health.decrease(50);
            }
        }
        commands.entity(entity).despawn();
    }
}

pub fn exhaust_blinking_system(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &MarkerRocketExhaust)>,
) {
    for (mut sprite, _) in query.iter_mut() {
        sprite.color.set_alpha(if time.elapsed_secs() % 1. < 0.85 { 1. } else { 0.0 });
    }
}