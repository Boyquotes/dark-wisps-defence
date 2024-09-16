use crate::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TechnicalState, TowerRange, TowerShootingTimer};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::wisps::components::Wisp;

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

pub const TOWER_EMITTER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 2, height: 2 };
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
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderTowerEmitter{ entity, grid_position } in events.read() {
            commands.entity(entity).insert((
                get_tower_emitter_sprite_bundle(&asset_server, grid_position),
                MarkerTower,
                MarkerTowerEmitter,
                grid_position,
                Health(10000),
                TowerRange(4),
                Building::from(BuildingType::Tower(TowerType::Emitter)),
                TowerShootingTimer::from_seconds(1.0),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, TOWER_EMITTER_GRID_IMPRINT) },
            ));
        }
    }
}
impl Command for BuilderTowerEmitter {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn get_tower_emitter_sprite_bundle(asset_server: &AssetServer, coords: GridCoords) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(TOWER_EMITTER_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_EMITTER_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(TOWER_EMITTER_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_emitters: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer), With<MarkerTowerEmitter>>,
    wisps: Query<(&GridPath, &GridCoords), With<Wisp>>,
) {
    // for (transform, technical_state, mut timer, mut target) in tower_emitters.iter_mut() {
    //     if !technical_state.has_energy_supply { continue; }
    //     let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
    //     if !timer.0.finished() { continue; }

    //     let Ok((wisp_grid_path, wisp_coords)) = wisps.get(*target_wisp) else {
    //         // Target wisp does not exist anymore
    //         *target = TowerWispTarget::SearchForNewTarget;
    //         continue;
    //     };

    //     // If wisps has path, target the next path position. Otherwise, target the wisp's current position.
    //     let target_world_position = wisp_grid_path.next_in_path().map_or(
    //         wisp_coords.to_world_position_centered(WISP_GRID_IMPRINT),
    //         |coords| coords.to_world_position_centered(WISP_GRID_IMPRINT)
    //     );

    //     commands.add(BuilderEmitterball::new(transform.translation.xy(), target_world_position));
    //     timer.0.reset();
    // }
}
