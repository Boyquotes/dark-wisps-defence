use lib_grid::grids::energy_supply::EnergySupplyGrid;

use crate::prelude::*;
use crate::projectiles::cannonball::BuilderCannonball;
use crate::wisps::components::Wisp;
use crate::wisps::spawning::WISP_GRID_IMPRINT;

pub struct TowerCannonPlugin;
impl Plugin for TowerCannonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderTowerCannon>()
            .add_systems(PostUpdate, (
                BuilderTowerCannon::spawn_system,
            )).add_systems(Update, (
                shooting_system.run_if(in_state(GameState::Running)),
            ));
    }
}

pub const TOWER_CANNON_BASE_IMAGE: &str = "buildings/tower_cannon.png";

#[derive(Component)]
pub struct MarkerTowerCannon;

#[derive(Event)]
pub struct BuilderTowerCannon {
    pub entity: Entity,
    pub grid_position: GridCoords,
}

impl BuilderTowerCannon {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self {
        Self { entity, grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderTowerCannon>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderTowerCannon{ entity, grid_position } in events.read() {
            let grid_imprint = almanach.get_building_info(BuildingType::Tower(TowerType::Cannon)).grid_imprint;
            commands.entity(entity).insert((
                Sprite {
                    image: asset_server.load(TOWER_CANNON_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                Transform::from_translation(grid_position.to_world_position_centered(grid_imprint).extend(Z_BUILDING)),
                MarkerTower,
                MarkerTowerCannon,
                grid_position,
                Health::new(100),
                TowerRange(15),
                Building,
                BuildingType::Tower(TowerType::Cannon),
                grid_imprint,
                TowerShootingTimer::from_seconds(2.0),
                TowerWispTarget::default(),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, grid_imprint), ..default() },
            ));
        }
    }
}
impl Command for BuilderTowerCannon {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_cannons: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget), With<MarkerTowerCannon>>,
    wisps: Query<(&GridPath, &GridCoords), With<Wisp>>,
) {
    for (transform, technical_state, mut timer, mut target) in tower_cannons.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok((wisp_grid_path, wisp_coords)) = wisps.get(target_wisp) else {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        // If wisps has path, target the next path position. Otherwise, target the wisp's current position.
        let target_world_position = wisp_grid_path.next_in_path().map_or(
            wisp_coords.to_world_position_centered(WISP_GRID_IMPRINT),
            |coords| coords.to_world_position_centered(WISP_GRID_IMPRINT)
        );

        commands.queue(BuilderCannonball::new(transform.translation.xy(), target_world_position));
        timer.0.reset();
    }
}
