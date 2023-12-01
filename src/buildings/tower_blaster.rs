use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TowerWispTarget, TowerShootingTimer, TechnicalState, TowerRange, TowerTopRotation, MarkerTowerRotationalTop};
use crate::common::{Z_BUILDING, Z_TOWER_TOP};
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::projectiles::laser_dart::create_laser_dart;
use crate::utils::math::angle_difference;
use crate::wisps::components::Wisp;

pub const TOWER_BLASTER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 2, height: 2 };
pub const TOWER_BLASTER_BASE_IMAGE: &str = "buildings/tower_blaster.png";

#[derive(Component)]
pub struct MarkerTowerBlaster;

pub fn create_tower_blaster(
    commands: &mut Commands,
    asset_server: &AssetServer,
    obstacle_grid: &mut ObstacleGrid,
    energy_supply_grid: &EnergySupplyGrid,
    grid_position: GridCoords
) -> Entity {
    let (tower_base_bundle, tower_top_bundle) = get_tower_blaster_sprite_bundle(grid_position, asset_server);
    let building_entity = commands.spawn((
        tower_base_bundle,
        MarkerTower,
        MarkerTowerBlaster,
        grid_position,
        Health(10000),
        TowerRange(15),
        Building::from(BuildingType::Tower(TowerType::Blaster)),
        TowerShootingTimer::from_seconds(0.2),
        TowerWispTarget::default(),
        TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, TOWER_BLASTER_GRID_IMPRINT) },
        TowerTopRotation { speed: 10.0, current_angle: 0. }
    )).id();

    // Spawn tower top
    commands.spawn((
        tower_top_bundle,
        MarkerTowerRotationalTop(building_entity.into()),
    ));
    obstacle_grid.imprint(grid_position, Field::Building(building_entity, BuildingType::Tower(TowerType::Blaster)), TOWER_BLASTER_GRID_IMPRINT);
    building_entity
}

/// Returns (tower base sprite bundle, tower top sprite bundle)
pub fn get_tower_blaster_sprite_bundle(grid_position: GridCoords, asset_server: &AssetServer) -> (SpriteBundle, SpriteBundle) {
    let world_position = grid_position.to_world_position_centered(TOWER_BLASTER_GRID_IMPRINT);
    let world_size = TOWER_BLASTER_GRID_IMPRINT.world_size();
    let tower_base = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(world_size),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_BLASTER_BASE_IMAGE),
        transform: Transform::from_translation(world_position.extend(Z_BUILDING)),
        ..Default::default()
    };

    let tower_top = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(world_size.x * 0.5, world_size.y * 1.52 * 0.5)),
            ..Default::default()
        },
        texture: asset_server.load("buildings/tower_blaster_top.png"),
        transform: Transform::from_translation(world_position.extend(Z_TOWER_TOP)),
        ..Default::default()
    };
    (tower_base, tower_top)
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_blasters: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget, &TowerTopRotation), With<MarkerTowerBlaster>>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (transform, technical_state, mut timer, mut target, top_rotation) in tower_blasters.iter_mut() {
        if !technical_state.is_operational() { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok(wisp_position) = wisps.get(*target_wisp).map(|target| target.translation.xy()) else {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        // Check if the tower top is facing the target
        let direction_to_target = wisp_position - transform.translation.xy();
        let target_angle = direction_to_target.y.atan2(direction_to_target.x);
        if angle_difference(target_angle, top_rotation.current_angle).abs() > std::f32::consts::PI / 36. { continue; }

        // Calculate transform offset in the direction we are aiming
        let tower_world_width = TOWER_BLASTER_GRID_IMPRINT.world_size().x;
        let offset = Vec2::new(
            top_rotation.current_angle.cos() * tower_world_width * 0.4,
            top_rotation.current_angle.sin() * tower_world_width * 0.4,
        );
        let spawn_position = transform.translation.xy() + offset;

        create_laser_dart(&mut commands, spawn_position, target_wisp, (wisp_position - spawn_position).normalize());
        timer.0.reset();
    }
}

pub fn rotating_system(
    time: Res<Time>,
    mut tower_blasters: Query<(&mut TowerTopRotation, &TowerWispTarget, &Transform), (With<MarkerTowerBlaster>, Without<Wisp>)>,
    wisps: Query<&Transform, (With<Wisp>, Without<MarkerTowerBlaster>)>,
) {
    for (mut rotation, target, tower_transform) in tower_blasters.iter_mut() {
        let TowerWispTarget::Wisp(target_wisp) = target else { continue; };
        let Ok(wisp_position) = wisps.get(**target_wisp).map(|target| target.translation.xy()) else { continue; };

        let direction_to_target = wisp_position - tower_transform.translation.xy();
        let target_angle = direction_to_target.y.atan2(direction_to_target.x);

        let angle_diff = angle_difference(target_angle, rotation.current_angle);

        let rotation_delta = rotation.speed * time.delta_seconds();
        rotation.current_angle += angle_diff.clamp(-rotation_delta, rotation_delta);
    }
}