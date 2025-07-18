use lib_grid::grids::emissions::{EmissionsType, EmitterEnergy};
use lib_grid::grids::energy_supply::{GeneratorEnergy, SupplierChangedEvent, SupplierEnergy};
use lib_grid::search::flooding::{FloodEmissionsDetails, FloodEmissionsEvaluator, FloodEmissionsMode, FloodEnergySupplyMode};

use crate::prelude::*;


pub struct MainBasePlugin;
impl Plugin for MainBasePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EventMoveMainBase>()
            .add_systems(Update, (
                move_main_base_system,
            ))
            .add_observer(BuilderMainBase::on_add);
    }
}

pub const MAIN_BASE_BASE_IMAGE: &str = "buildings/main_base.png";


#[derive(Component)]
pub struct BuilderMainBase {
    grid_position: GridCoords,
}
impl BuilderMainBase {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
     }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderMainBase>,
        mut commands: Commands,
        builders: Query<&BuilderMainBase>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let grid_imprint = almanach.get_building_info(BuildingType::MainBase).grid_imprint;
        commands.entity(entity)
            .remove::<BuilderMainBase>()
            .insert((
                Sprite {
                    image: asset_server.load(MAIN_BASE_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                Transform::from_translation(builder.grid_position.to_world_position_centered(grid_imprint).extend(Z_BUILDING)),
                MainBase,
                builder.grid_position,
                Health::new(10000),
                Building,
                BuildingType::MainBase,
                grid_imprint,
                EmitterEnergy(FloodEmissionsDetails {
                    emissions_type: EmissionsType::Energy,
                    range: usize::MAX,
                    evaluator: FloodEmissionsEvaluator::ExponentialDecay { start_value: 100., decay: 0.1 },
                    mode: FloodEmissionsMode::Increase,
                }),
                GeneratorEnergy,
                SupplierEnergy { range: 15 },
                TechnicalState { has_energy_supply: true, ..default() },
            ));
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
    mut commands: Commands,
    mut events: EventReader<EventMoveMainBase>,
    mut main_base: Query<(Entity, &GridImprint, &mut Transform), With<MainBase>>,
) {
    for &EventMoveMainBase { new_grid_position } in events.read() {
        let Ok((entity, grid_imprint, mut transform)) = main_base.single_mut() else { return; };
        transform.translation = new_grid_position.to_world_position_centered(*grid_imprint).extend(Z_BUILDING);
        commands.entity(entity).insert(new_grid_position);
    }
}
