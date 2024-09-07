use crate::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TechnicalState, TowerRange, TowerShootingTimer, TowerWispTarget};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::projectiles::cannonball::BuilderCannonball;
use crate::wisps::components::{Target, Wisp};
use crate::wisps::spawning::WISP_GRID_IMPRINT;

pub struct TowerCannonPlugin;
impl Plugin for TowerCannonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderTowerCannon>()
            .add_systems(PostUpdate, (
                BuilderTowerCannon::spawn_system,
            )).add_systems(Update, (
                shooting_system,
            ));
    }
}

pub const TOWER_CANNON_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };
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
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderTowerCannon{ entity, grid_position } in events.read() {
            commands.entity(entity).insert((
                get_tower_cannon_sprite_bundle(&asset_server, grid_position),
                MarkerTower,
                MarkerTowerCannon,
                grid_position,
                Health(10000),
                TowerRange(15),
                Building::from(BuildingType::Tower(TowerType::Cannon)),
                TowerShootingTimer::from_seconds(2.0),
                TowerWispTarget::default(),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, TOWER_CANNON_GRID_IMPRINT) },
            ));
        }
    }
}
impl Command for BuilderTowerCannon {
    fn apply(self, world: &mut World) {
        world.send_event(self);
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

        commands.add(BuilderCannonball::new(transform.translation.xy(), target_world_position));
        timer.0.reset();
    }
}
