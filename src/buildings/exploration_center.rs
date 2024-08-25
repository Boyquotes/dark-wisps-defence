use crate::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, TechnicalState};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::map_objects::common::ExpeditionZone;
use crate::units::expedition_drone::BuilderExpeditionDrone;

pub const EXPLORATION_CENTER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 4, height: 4 };
pub const EXPLORATION_CENTER_BASE_IMAGE: &str = "buildings/exploration_center.png";


#[derive(Component)]
pub struct MarkerExplorationCenter;

#[derive(Component)]
pub struct ExplorationCenterNewExpeditionTimer(pub Timer);

#[derive(Bundle)]
pub struct BuilderExplorationCenter {
    pub sprite: SpriteBundle,
    pub marker_exploration_center: MarkerExplorationCenter,
    pub grid_position: GridCoords,
    pub health: Health,
    pub building: Building,
    pub technical_state: TechnicalState,
    pub exploration_center_delivery_timer: ExplorationCenterNewExpeditionTimer,
}
impl BuilderExplorationCenter {
    pub fn new(grid_position: GridCoords, asset_server: &AssetServer) -> Self {
        Self {
            sprite: get_exploration_center_sprite_bundle(asset_server, grid_position),
            marker_exploration_center: MarkerExplorationCenter,
            grid_position,
            health: Health(10000),
            building: Building::from(BuildingType::ExplorationCenter),
            technical_state: TechnicalState::default(),
            exploration_center_delivery_timer: ExplorationCenterNewExpeditionTimer(Timer::from_seconds(3.0, TimerMode::Repeating)),
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

pub fn create_expedition_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    //mut dark_ore_stock: ResMut<DarkOreStock>,
    mut exploration_centres: Query<(&mut ExplorationCenterNewExpeditionTimer, &TechnicalState, &Transform), With<MarkerExplorationCenter>>,
    expedition_zones: Query<(Entity, &Transform), With<ExpeditionZone>>,
    time: Res<Time>,
) {
    let mut zones_positions = None;
    for (mut timer, technical_state, exploration_center_transform) in exploration_centres.iter_mut() {
        if !technical_state.is_operational() { continue; }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            // Just caching to avoid recomputing every frame when there is no expeditions to create
            if zones_positions.is_none() {
                zones_positions = Some(expedition_zones.iter().map(|(entity, transform)| (entity, transform.translation.xy())).collect::<Vec<_>>());
            }
            let center_position = exploration_center_transform.translation.xy();
            let closest_zone = zones_positions.as_ref().unwrap().iter().min_by(|a, b| {
                a.1.distance_squared(center_position).total_cmp(&b.1.distance_squared(center_position))
            });
            if let Some((zone_entity, zone_position)) = closest_zone {
                BuilderExpeditionDrone::new(center_position, &asset_server)
                    .with_target(*zone_entity, *zone_position)
                    .spawn(&mut commands);
            }
        }
    }
}
