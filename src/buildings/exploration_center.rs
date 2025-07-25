use lib_grid::grids::energy_supply::EnergySupplyGrid;

use crate::prelude::*;
use crate::map_objects::common::{ExpeditionTargetMarker, ExpeditionZone};
use crate::units::expedition_drone::BuilderExpeditionDrone;

pub struct ExplorationCenterPlugin;
impl Plugin for ExplorationCenterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                create_expedition_system.run_if(in_state(GameState::Running)),
            ))
            .add_observer(BuilderExplorationCenter::on_add);
    }
}

pub const EXPLORATION_CENTER_BASE_IMAGE: &str = "buildings/exploration_center.png";


#[derive(Component)]
pub struct ExplorationCenterNewExpeditionTimer(pub Timer);

#[derive(Component)]
pub struct BuilderExplorationCenter {
    grid_position: GridCoords,
}
impl BuilderExplorationCenter {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderExplorationCenter>,
        mut commands: Commands,
        builders: Query<&BuilderExplorationCenter>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let grid_imprint = almanach.get_building_info(BuildingType::ExplorationCenter).grid_imprint;
        commands.entity(entity)
            .remove::<BuilderExplorationCenter>()
            .insert((
                Sprite {
                    image: asset_server.load(EXPLORATION_CENTER_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                ExplorationCenter,
                builder.grid_position,
                Health::new(100),
                grid_imprint,
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(builder.grid_position, grid_imprint), ..default() },
                ExplorationCenterNewExpeditionTimer(Timer::from_seconds(3.0, TimerMode::Repeating)),
            ));
    }
}

pub fn create_expedition_system(
    mut commands: Commands,
    //mut dark_ore_stock: ResMut<DarkOreStock>,
    mut exploration_centres: Query<(&mut ExplorationCenterNewExpeditionTimer, &TechnicalState, &Transform), With<ExplorationCenter>>,
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
                commands.spawn(BuilderExpeditionDrone::new(center_position, *zone_entity));
            }
        }
    }
}
