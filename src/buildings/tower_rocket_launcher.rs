use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, MarkerTowerRotationalTop, TechnicalState, TowerRange, TowerShootingTimer, TowerTopRotation, TowerWispTarget};
use crate::buildings::tower_blaster::TOWER_BLASTER_GRID_IMPRINT;
use crate::common::{Z_BUILDING, Z_TOWER_TOP};
use crate::common_components::{Health};
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::projectiles::rocket::create_rocket;
use crate::utils::math::angle_difference;
use crate::wisps::components::{Target, Wisp};

pub const TOWER_ROCKET_LAUNCHER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };

pub const TOWER_ROCKET_LAUNCHER_BASE_IMAGE: &str = "buildings/tower_rocket_launcher.png";

#[derive(Component)]
pub struct MarkerTowerRocketLauncher;

pub fn create_tower_rocket_launcher(
    commands: &mut Commands,
    asset_server: &AssetServer,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    energy_supply_grid: &EnergySupplyGrid,
    grid_position: GridCoords,
) -> Entity {
    let (tower_base_bundle, tower_top_bundle) = get_tower_rocket_launcher_sprite_bundle(asset_server, grid_position);
    let building_entity = commands.spawn((
        tower_base_bundle,
        MarkerTower,
        MarkerTowerRocketLauncher,
        grid_position,
        Health(10000),
        TowerRange(30),
        Building::from(BuildingType::Tower(TowerType::RocketLauncher)),
        TowerShootingTimer::from_seconds(2.0),
        TowerWispTarget::default(),
        TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, TOWER_ROCKET_LAUNCHER_GRID_IMPRINT) },
        TowerTopRotation { speed: 1.0, current_angle: 0. },
    )).id();
    // Spawn tower top
    commands.spawn((
        tower_top_bundle,
        MarkerTowerRotationalTop(building_entity.into()),
    ));
    obstacle_grid.imprint(grid_position, Field::Building(building_entity, BuildingType::Tower(TowerType::RocketLauncher)), TOWER_ROCKET_LAUNCHER_GRID_IMPRINT);
    building_entity
}

pub fn get_tower_rocket_launcher_sprite_bundle(asset_server: &AssetServer, coords: GridCoords) -> (SpriteBundle, SpriteBundle) {
    let world_position = coords.to_world_position_centered(TOWER_ROCKET_LAUNCHER_GRID_IMPRINT);
    let world_size = TOWER_ROCKET_LAUNCHER_GRID_IMPRINT.world_size();
    let tower_base = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(world_size),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_ROCKET_LAUNCHER_BASE_IMAGE),
        transform: Transform::from_translation(world_position.extend(Z_BUILDING)),
        ..Default::default()
    };
    let tower_top = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(world_size.x * 0.5, world_size.y * 1.52 * 0.5)),
            anchor: Anchor::Custom(Vec2::new(0., 0.20)),
            ..Default::default()
        },
        texture: asset_server.load("buildings/tower_rocket_launcher_top.png"),
        transform: Transform::from_translation(world_position.extend(Z_TOWER_TOP)),
        ..Default::default()
    };
    (tower_base, tower_top)
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_rocket_launchers: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget, &TowerTopRotation), (With<MarkerTowerRocketLauncher>, Without<Wisp>)>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (transform, technical_state, mut timer, mut target, top_rotation) in tower_rocket_launchers.iter_mut() {
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
        if angle_difference(target_angle, top_rotation.current_angle).abs() > std::f32::consts::PI / 72. { continue; }

        // Calculate transform offset in the direction we are aiming
        let tower_world_width = TOWER_BLASTER_GRID_IMPRINT.world_size().x;
        let offset = Vec2::new(
            top_rotation.current_angle.cos() * tower_world_width * 0.4,
            top_rotation.current_angle.sin() * tower_world_width * 0.4,
        );
        let spawn_position = transform.translation.xy() + offset;

        create_rocket(&mut commands, spawn_position, target_wisp);
        timer.0.reset();
    }
}
