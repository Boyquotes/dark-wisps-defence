use crate::prelude::*;
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::map_objects::common::{ExpeditionTargetMarker, ExpeditionZone};
use crate::units::expedition_drone::BuilderExpeditionDrone;

pub struct ExplorationCenterPlugin;
impl Plugin for ExplorationCenterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderExplorationCenter>()
            .add_systems(PostUpdate, (
                BuilderExplorationCenter::spawn_system,
            )).add_systems(Update, (
                create_expedition_system.run_if(in_state(GameState::Running)),
            ));
    }
}

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
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderExplorationCenter{ entity, grid_position } in events.read() {
            let grid_imprint = almanach.get_building_grid_imprint(BuildingType::ExplorationCenter);
            commands.entity(entity).insert((
                Sprite {
                    image: asset_server.load(EXPLORATION_CENTER_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                Transform::from_translation(grid_position.to_world_position_centered(grid_imprint).extend(Z_BUILDING)),
                MarkerExplorationCenter,
                grid_position,
                Health::new(100),
                Building,
                BuildingType::ExplorationCenter,
                grid_imprint,
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, grid_imprint), ..default() },
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

pub fn create_expedition_system(
    mut commands: Commands,
    //mut dark_ore_stock: ResMut<DarkOreStock>,
    mut exploration_centres: Query<(&mut ExplorationCenterNewExpeditionTimer, &TechnicalState, &Transform), With<MarkerExplorationCenter>>,
    expedition_zones: Query<(Entity, &Transform), (With<ExpeditionZone>, With<ExpeditionTargetMarker>)>,
    time: Res<Time>,
) {
    let mut zones_positions = None;
    for (mut timer, technical_state, exploration_center_transform) in exploration_centres.iter_mut() {
        if !technical_state.is_operational() { continue; }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if zones_positions.is_none() {
                // Cache to avoid recomputing zone positions for every exploration center
                zones_positions = Some(expedition_zones.iter().map(|(entity, transform)| (entity, transform.translation.xy())).collect::<Vec<_>>());
            }
            let center_position = exploration_center_transform.translation.xy();
            let closest_zone = zones_positions.as_ref().unwrap().iter().min_by(|a, b| {
                a.1.distance_squared(center_position).total_cmp(&b.1.distance_squared(center_position))
            });
            if let Some((zone_entity, ..)) = closest_zone {
                let drone_entity = commands.spawn_empty().id();
                commands.queue(BuilderExpeditionDrone::new(drone_entity, center_position, *zone_entity));
            }
        }
    }
}
