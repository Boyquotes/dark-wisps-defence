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
use crate::wisps::components::{Wisp};

pub const TOWER_ROCKET_LAUNCHER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };

pub const TOWER_ROCKET_LAUNCHER_BASE_IMAGE: &str = "buildings/tower_rocket_launcher.png";

#[derive(Component)]
pub struct MarkerTowerRocketLauncher;

#[derive(Bundle)]
pub struct BundleTowerRocketLauncherBase {
    pub sprite: SpriteBundle,
    pub marker_tower: MarkerTower,
    pub marker_tower_rocket_launcher: MarkerTowerRocketLauncher,
    pub grid_position: GridCoords,
    pub health: Health,
    pub tower_range: TowerRange,
    pub building: Building,
    pub tower_shooting_timer: TowerShootingTimer,
    pub tower_wisp_target: TowerWispTarget,
    pub technical_state: TechnicalState,
    pub tower_top_rotation: TowerTopRotation,
}
#[derive(Bundle)]
pub struct BundleTowerRocketLauncherTop {
    pub sprite: SpriteBundle,
    pub marker: MarkerTowerRotationalTop,
}
pub struct BundleTowerRocketLauncher {
    pub base: BundleTowerRocketLauncherBase,
    pub top: BundleTowerRocketLauncherTop,
}
impl BundleTowerRocketLauncher {
    pub fn new(grid_position: GridCoords, asset_server: &AssetServer) -> Self {
        let (tower_base, tower_top) = get_tower_rocket_launcher_sprite_bundle(asset_server, grid_position);
        Self {
            base: BundleTowerRocketLauncherBase {
                sprite: tower_base,
                marker_tower: MarkerTower,
                marker_tower_rocket_launcher: MarkerTowerRocketLauncher,
                grid_position,
                health: Health(10000),
                tower_range: TowerRange(30),
                building: Building::from(BuildingType::Tower(TowerType::RocketLauncher)),
                tower_shooting_timer: TowerShootingTimer::from_seconds(2.0),
                tower_wisp_target: TowerWispTarget::default(),
                technical_state: TechnicalState::default(),
                tower_top_rotation: TowerTopRotation { speed: 1.0, current_angle: 0. },
            },
            top: BundleTowerRocketLauncherTop {
                sprite: tower_top,
                marker: MarkerTowerRotationalTop::default(),
            },
        }
    }
    pub fn update_energy_supply(mut self, energy_supply_grid: &EnergySupplyGrid) -> Self {
        self.base.technical_state.has_energy_supply = energy_supply_grid.is_imprint_suppliable(self.base.grid_position, TOWER_ROCKET_LAUNCHER_GRID_IMPRINT);
        self
    }
    pub fn spawn(self, commands: &mut Commands, obstacle_grid: &mut ResMut<ObstacleGrid>, ) -> Entity {
        let grid_position = self.base.grid_position;
        let BundleTowerRocketLauncher{ base, mut top } = self;
        let base_entity = commands.spawn(base).id();
        top.marker.0 = base_entity.into(); // Set parent reference
        let _ = commands.spawn(top).id();
        obstacle_grid.imprint(grid_position, Field::Building(base_entity, BuildingType::Tower(TowerType::RocketLauncher)), TOWER_ROCKET_LAUNCHER_GRID_IMPRINT);
        base_entity
    }

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
            custom_size: Some(Vec2::new(world_size.x * 1.52 * 0.5, world_size.y * 0.5)),
            anchor: Anchor::Custom(Vec2::new(-0.20, 0.0)),
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

        let rocket_angle = Quat::from_rotation_z(top_rotation.current_angle);
        create_rocket(&mut commands, spawn_position, rocket_angle, target_wisp);
        timer.0.reset();
    }
}
