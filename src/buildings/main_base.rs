use crate::prelude::*;
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

pub const MAIN_BASE_BASE_IMAGE: &str = "buildings/main_base.png";

#[derive(Component)]
pub struct MarkerMainBase;


#[derive(Event)]
pub struct BuilderMainBase {
    pub entity: Entity,
    pub grid_position: GridCoords,
}
impl BuilderMainBase {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self {
        Self { entity, grid_position }
     }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderMainBase>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        mut emitter_created_event_writer: EventWriter<EmitterChangedEvent>,
        mut supplier_created_event_writer: EventWriter<SupplierChangedEvent>,
    ) {
        for &BuilderMainBase { entity, grid_position } in events.read() {
            let grid_imprint = almanach.get_building_grid_imprint(BuildingType::MainBase);
            let covered_coords = grid_imprint.covered_coords(grid_position);
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
                get_building_sprite_bundle(&asset_server, MAIN_BASE_BASE_IMAGE, grid_position, grid_imprint),
                MarkerMainBase,
                grid_position,
                Health(10000),
                Building,
                BuildingType::MainBase,
                grid_imprint,
                emitter_energy,
                supplier_energy,
                TechnicalState { has_energy_supply: true, ..default() },
            ));
        }
    }
}
impl Command for BuilderMainBase {
    fn apply(self, world: &mut World) {
        world.send_event(self);
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
    mut main_base: Query<(Entity, &GridImprint, &mut GridCoords, &SupplierEnergy, &mut Transform), With<MarkerMainBase>>,
) {
    for &EventMoveMainBase { new_grid_position } in events.read() {
        let (entity, grid_imprint, mut main_base_location, supplier_energy, mut transform) = main_base.single_mut();
        supplier_created_event_writer.send(SupplierChangedEvent {
            coords: grid_imprint.covered_coords(*main_base_location),
            supplier: supplier_energy.clone(),
            mode: FloodEnergySupplyMode::Decrease,
        });
        supplier_created_event_writer.send(SupplierChangedEvent {
            coords: grid_imprint.covered_coords(new_grid_position),
            supplier: supplier_energy.clone(),
            mode: FloodEnergySupplyMode::Increase,
        });
        emitter_created_event_writer.send(EmitterChangedEvent {
            emitter_entity: entity,
            coords: grid_imprint.covered_coords(*main_base_location),
            emissions_details: vec![FloodEmissionsDetails {
                emissions_type: EmissionsType::Energy,
                range: usize::MAX,
                evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
                mode: FloodEmissionsMode::Decrease,
            }]
        });
        emitter_created_event_writer.send(EmitterChangedEvent {
            emitter_entity: entity,
            coords: grid_imprint.covered_coords(new_grid_position),
            emissions_details: vec![FloodEmissionsDetails {
                emissions_type: EmissionsType::Energy,
                range: usize::MAX,
                evaluator: FloodEmissionsEvaluator::ExponentialDecay{start_value: 100., decay: 0.1},
                mode: FloodEmissionsMode::Increase,
            }]
        });
        *main_base_location = new_grid_position;
        transform.translation = new_grid_position.to_world_position_centered(*grid_imprint).extend(Z_BUILDING);
    }
}
