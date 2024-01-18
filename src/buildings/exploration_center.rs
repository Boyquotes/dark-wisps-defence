use bevy::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, TechnicalState};
use crate::common::Z_BUILDING;
use crate::common_components::Health;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};

pub const EXPLORATION_CENTER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 4, height: 4 };
pub const EXPLORATION_CENTER_BASE_IMAGE: &str = "buildings/exploration_center.png";


#[derive(Component)]
pub struct MarkerExplorationCenter;

#[derive(Component)]
pub struct ExplorationCenterDeliveryTimer(pub Timer);

#[derive(Bundle)]
pub struct BundleExplorationCenter {
    pub sprite: SpriteBundle,
    pub marker_exploration_center: MarkerExplorationCenter,
    pub grid_position: GridCoords,
    pub health: Health,
    pub building: Building,
    pub technical_state: TechnicalState,
    pub exploration_center_delivery_timer: ExplorationCenterDeliveryTimer,
}
impl BundleExplorationCenter {
    pub fn new(grid_position: GridCoords, asset_server: &AssetServer) -> Self {
        Self {
            sprite: get_exploration_center_sprite_bundle(asset_server, grid_position),
            marker_exploration_center: MarkerExplorationCenter,
            grid_position,
            health: Health(10000),
            building: Building::from(BuildingType::ExplorationCenter),
            technical_state: TechnicalState::default(),
            exploration_center_delivery_timer: ExplorationCenterDeliveryTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        }
    }
    pub fn update_energy_supply(mut self, energy_supply_grid: &EnergySupplyGrid) -> Self {
        self.technical_state.has_energy_supply = energy_supply_grid.is_imprint_suppliable(self.grid_position, EXPLORATION_CENTER_GRID_IMPRINT);
        self
    }
    pub fn spawn(self, commands: &mut Commands, obstacle_grid: &mut ObstacleGrid) -> Entity {
        let grid_position = self.grid_position;
        let base_entity = commands.spawn(self).id();
        obstacle_grid.imprint(grid_position, Field::Building(base_entity, BuildingType::ExplorationCenter), EXPLORATION_CENTER_GRID_IMPRINT);
        base_entity
    }
}

pub fn get_exploration_center_sprite_bundle(asset_server: &AssetServer, coords: GridCoords) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(EXPLORATION_CENTER_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(EXPLORATION_CENTER_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(EXPLORATION_CENTER_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}

// pub fn mine_ore_system(
//     mut dark_ore_stock: ResMut<DarkOreStock>,
//     mut exploration_centeres: Query<(&mut ExplorationCenterDeliveryTimer, &TechnicalState), With<MarkerExplorationCenter>>,
//     time: Res<Time>,
// ) {
//     for (mut timer, technical_state) in exploration_centeres.iter_mut() {
//         if !technical_state.is_operational() { continue; }
//         timer.0.tick(time.delta());
//         if timer.0.just_finished() {
//             dark_ore_stock.add(10);
//         }
//     }
// }
