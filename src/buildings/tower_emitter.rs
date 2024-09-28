use crate::effects::ripple::BuilderRipple;
use crate::prelude::*;
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::wisps::components::Wisp;

use super::common_components::TowerWispTarget;

pub struct TowerEmitterPlugin;
impl Plugin for TowerEmitterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderTowerEmitter>()
            .add_systems(PostUpdate, (
                BuilderTowerEmitter::spawn_system,
            )).add_systems(Update, (
                shooting_system,
            ));
    }
}

pub const TOWER_EMITTER_BASE_IMAGE: &str = "buildings/tower_emitter.png";

#[derive(Component)]
pub struct MarkerTowerEmitter;

#[derive(Event)]
pub struct BuilderTowerEmitter {
    pub entity: Entity,
    pub grid_position: GridCoords,
}

impl BuilderTowerEmitter {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self {
        Self { entity, grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderTowerEmitter>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderTowerEmitter{ entity, grid_position } in events.read() {
            let grid_imprint = almanach.get_building_grid_imprint(BuildingType::Tower(TowerType::Emitter));
            commands.entity(entity).insert((
                get_building_sprite_bundle(&asset_server, TOWER_EMITTER_BASE_IMAGE, grid_position, grid_imprint),
                MarkerTower,
                MarkerTowerEmitter,
                grid_position,
                Health::new(100),
                TowerRange(4),
                Building,
                BuildingType::Tower(TowerType::Emitter),
                grid_imprint,
                TowerShootingTimer::from_seconds(2.0),
                TowerWispTarget::default(),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, grid_imprint), ..default() },
            ));
        }
    }
}
impl Command for BuilderTowerEmitter {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_emitters: Query<(&Transform, &TechnicalState, &TowerRange, &mut TowerShootingTimer, &mut TowerWispTarget), With<MarkerTowerEmitter>>,
    wisps: Query<(), With<Wisp>>,
) {
    for (transform, technical_state, range, mut timer, mut target) in tower_emitters.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        if !wisps.contains(*target_wisp) {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        commands.add(BuilderRipple::new(transform.translation.xy(), range.0 as f32 * CELL_SIZE));
        timer.0.reset();
    }
}
