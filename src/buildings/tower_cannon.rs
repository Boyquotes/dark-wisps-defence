use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TechnicalState, TowerRange, TowerShootingTimer, TowerWispTarget};
use crate::common::Z_BUILDING;
use crate::common_components::{Health};
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::projectiles::cannonball::BuilderCannonball;
use crate::wisps::components::{Target, Wisp};
use crate::wisps::spawning::WISP_GRID_IMPRINT;
pub const TOWER_CANNON_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };
pub const TOWER_CANNON_BASE_IMAGE: &str = "buildings/tower_cannon.png";

#[derive(Component)]
pub struct MarkerTowerCannon;

#[derive(Bundle)]
pub struct BuilderTowerCannon {
    pub sprite: SpriteBundle,
    pub marker_tower: MarkerTower,
    pub marker_tower_cannon: MarkerTowerCannon,
    pub grid_position: GridCoords,
    pub health: Health,
    pub tower_range: TowerRange,
    pub building: Building,
    pub tower_shooting_timer: TowerShootingTimer,
    pub tower_wisp_target: TowerWispTarget,
    pub technical_state: TechnicalState,
}

impl BuilderTowerCannon {
    pub fn new(grid_position: GridCoords, asset_server: &AssetServer) -> Self {
        Self {
            sprite: get_tower_cannon_sprite_bundle(asset_server, grid_position),
            marker_tower: MarkerTower,
            marker_tower_cannon: MarkerTowerCannon,
            grid_position,
            health: Health(10000),
            tower_range: TowerRange(15),
            building: Building::from(BuildingType::Tower(TowerType::Cannon)),
            tower_shooting_timer: TowerShootingTimer::from_seconds(2.0),
            tower_wisp_target: TowerWispTarget::default(),
            technical_state: TechnicalState::default(),
        }
    }
    pub fn update_energy_supply(mut self, energy_supply_grid: &EnergySupplyGrid) -> Self {
        self.technical_state.has_energy_supply = energy_supply_grid.is_imprint_suppliable(self.grid_position, TOWER_CANNON_GRID_IMPRINT);
        self
    }
    pub fn spawn(self, commands: &mut Commands, obstacle_grid: &mut ObstacleGrid) -> Entity {
        let grid_position = self.grid_position;
        let base_entity = commands.spawn(self).id();
        obstacle_grid.imprint(grid_position, Field::Building(base_entity, BuildingType::Tower(TowerType::Cannon)), TOWER_CANNON_GRID_IMPRINT);
        base_entity
    }
}

pub fn get_tower_cannon_sprite_bundle(asset_server: &AssetServer, coords: GridCoords) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(TOWER_CANNON_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_CANNON_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(TOWER_CANNON_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}

pub fn shooting_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tower_cannons: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget), With<MarkerTowerCannon>>,
    wisps: Query<(&Target, &GridCoords), With<Wisp>>,
) {
    for (transform, technical_state, mut timer, mut target) in tower_cannons.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok((wisp_target, wisp_coords)) = wisps.get(*target_wisp) else {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        let target_world_position = wisp_target.grid_path
            .as_ref()
            .map_or(
                wisp_coords.to_world_position_centered(WISP_GRID_IMPRINT),
                |path| {
                    path
                        .first()
                        .map_or(
                            wisp_coords.to_world_position_centered(WISP_GRID_IMPRINT),
                            |coords| coords.to_world_position_centered(WISP_GRID_IMPRINT)
                        )
                }
            );

        BuilderCannonball::new(transform.translation.xy(), target_world_position, &asset_server).spawn(&mut commands);
        timer.0.reset();
    }
}
