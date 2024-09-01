use crate::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, TechnicalState};
use crate::grids::emissions::{EmissionsType, EmitterChangedEvent, EmitterEnergy};
use crate::grids::energy_supply::{SupplierChangedEvent, SupplierEnergy};
use crate::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode, FloodEnergySupplyMode};

pub struct MainBasePlugin;
impl Plugin for MainBasePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderMainBase>()
            .add_event::<EventMoveMainBase>()
            .add_systems(PostUpdate, (
                BuilderMainBase::spawn_system,
            )).add_systems(Update, (
                move_main_base_system,
            ));
    }
}

pub const MAIN_BASE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 6, height: 6 };
pub const MAIN_BASE_BASE_IMAGE: &str = "buildings/main_base.png";

#[derive(Component)]
pub struct MarkerMainBase;


#[derive(Event)]
pub struct BuilderMainBase {
    pub entity: LazyEntity,
    pub grid_position: GridCoords,
}
impl BuilderMainBase {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { entity: LazyEntity::default(), grid_position }
     }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderMainBase>,
        asset_server: Res<AssetServer>,
        mut emitter_created_event_writer: EventWriter<EmitterChangedEvent>,
        mut supplier_created_event_writer: EventWriter<SupplierChangedEvent>,
    ) {
        for &BuilderMainBase { mut entity, grid_position } in events.read() {
            let entity = entity.get(&mut commands);
            let covered_coords = MAIN_BASE_GRID_IMPRINT.covered_coords(grid_position);
            let emitter_energy = EmitterEnergy(FloodEmissionsDetails {
                emissions_type: EmissionsType::Energy,
                range: usize::MAX,
                evaluator: FloodEmissionsEvaluator::ExponentialDecay { start_value: 100., decay: 0.1 },
                mode: FloodEmissionsMode::Increase,
            });
            emitter_created_event_writer.send(EmitterChangedEvent {
                emitter_entity: entity,
                coords: covered_coords.clone(),
                emissions_details: vec![emitter_energy.0.clone()],
            });
            let supplier_energy = SupplierEnergy { range: 15 };
            supplier_created_event_writer.send(SupplierChangedEvent {
                coords: covered_coords,
                supplier: supplier_energy.clone(),
                mode: FloodEnergySupplyMode::Increase,
            });

            commands.entity(entity).insert((
                get_main_base_sprite_bundle(grid_position, &asset_server),
                MarkerMainBase,
                grid_position,
                Health(10000),
                Building::from(BuildingType::MainBase),
                emitter_energy,
                supplier_energy,
                TechnicalState { has_energy_supply: true },
            ));
        }
    }
}
impl Command for BuilderMainBase {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn get_main_base_sprite_bundle(coords: GridCoords, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(MAIN_BASE_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(MAIN_BASE_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(MAIN_BASE_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}


#[derive(Event)]
pub struct EventMoveMainBase {
    pub new_grid_position: GridCoords,
}
impl Command for EventMoveMainBase {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}
pub fn move_main_base_system(
    mut events: EventReader<EventMoveMainBase>,
    mut emitter_created_event_writer: EventWriter<EmitterChangedEvent>,
    mut supplier_created_event_writer: EventWriter<SupplierChangedEvent>,
    mut main_base: Query<(Entity, &mut GridCoords, &SupplierEnergy, &mut Transform), With<MarkerMainBase>>,
) {
    for &EventMoveMainBase { new_grid_position } in events.read() {
        let (entity, mut main_base_location, supplier_energy, mut transform) = main_base.single_mut();
        supplier_created_event_writer.send(SupplierChangedEvent {
            coords: MAIN_BASE_GRID_IMPRINT.covered_coords(*main_base_location),
            supplier: supplier_energy.clone(),
            mode: FloodEnergySupplyMode::Decrease,
        });
        supplier_created_event_writer.send(SupplierChangedEvent {
            coords: MAIN_BASE_GRID_IMPRINT.covered_coords(new_grid_position),
            supplier: supplier_energy.clone(),
            mode: FloodEnergySupplyMode::Increase,
        });
        emitter_created_event_writer.send(EmitterChangedEvent {
            emitter_entity: entity,
            coords: MAIN_BASE_GRID_IMPRINT.covered_coords(*main_base_location),
            emissions_details: vec![FloodEmissionsDetails {
                emissions_type: EmissionsType::Energy,
                range: usize::MAX,
                evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
                mode: FloodEmissionsMode::Decrease,
            }]
        });
        emitter_created_event_writer.send(EmitterChangedEvent {
            emitter_entity: entity,
            coords: MAIN_BASE_GRID_IMPRINT.covered_coords(new_grid_position),
            emissions_details: vec![FloodEmissionsDetails {
                emissions_type: EmissionsType::Energy,
                range: usize::MAX,
                evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
                mode: FloodEmissionsMode::Increase,
            }]
        });
        *main_base_location = new_grid_position;
        transform.translation = new_grid_position.to_world_position_centered(MAIN_BASE_GRID_IMPRINT).extend(Z_BUILDING);
    }
}
