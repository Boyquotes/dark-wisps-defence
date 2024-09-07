use crate::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, TechnicalState};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::map_objects::common::ExpeditionZone;
use crate::units::expedition_drone::BuilderExpeditionDrone;

pub struct ExplorationCenterPlugin;
impl Plugin for ExplorationCenterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderExplorationCenter>()
            .add_systems(PostUpdate, (
                BuilderExplorationCenter::spawn_system,
            )).add_systems(Update, (
                create_expedition_system,
            ));
    }
}

pub const EXPLORATION_CENTER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 4, height: 4 };
pub const EXPLORATION_CENTER_BASE_IMAGE: &str = "buildings/exploration_center.png";


#[derive(Component)]
pub struct MarkerExplorationCenter;

#[derive(Component)]
pub struct ExplorationCenterNewExpeditionTimer(pub Timer);

#[derive(Event)]
pub struct BuilderExplorationCenter {
    pub entity: Entity,
    pub grid_position: GridCoords,
}
impl BuilderExplorationCenter {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self {
        Self { entity, grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderExplorationCenter>,
        asset_server: Res<AssetServer>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderExplorationCenter{ entity, grid_position } in events.read() {
            commands.entity(entity).insert((
                get_exploration_center_sprite_bundle(&asset_server, grid_position),
                MarkerExplorationCenter,
                grid_position,
                Health(10000),
                Building::from(BuildingType::ExplorationCenter),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, EXPLORATION_CENTER_GRID_IMPRINT) },
                ExplorationCenterNewExpeditionTimer(Timer::from_seconds(3.0, TimerMode::Repeating)),
            ));
        }
    }
}
impl Command for BuilderExplorationCenter {
    fn apply(self, world: &mut World) {
        world.send_event(self);
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
            if let Some((zone_entity, ..)) = closest_zone {
                let drone_entity = commands.spawn_empty().id();
                commands.add(BuilderExpeditionDrone::new(drone_entity, center_position, *zone_entity));
            }
        }
    }
}
