use std::fs::File;

use lib_grid::grids::emissions::EmissionsGrid;
use lib_grid::grids::energy_supply::EnergySupplyGrid;
use lib_grid::grids::obstacles::{Field, ObstacleGrid};
use lib_grid::grids::tower_ranges::TowerRangesGrid;
use lib_grid::grids::wisps::WispsGrid;

use crate::map_editor::MapInfo;
use crate::prelude::*;
use crate::buildings::energy_relay::BuilderEnergyRelay;
use crate::buildings::exploration_center::BuilderExplorationCenter;
use crate::buildings::main_base::BuilderMainBase;
use crate::buildings::tower_blaster::BuilderTowerBlaster;
use crate::buildings::tower_cannon::BuilderTowerCannon;
use crate::buildings::tower_emitter::BuilderTowerEmitter;
use crate::buildings::tower_rocket_launcher::BuilderTowerRocketLauncher;
use crate::objectives::ObjectiveDetails;
use crate::map_objects::dark_ore::{BuilderDarkOre, DARK_ORE_GRID_IMPRINT};
use crate::map_objects::quantum_field::BuilderQuantumField;
use crate::map_objects::walls::{BuilderWall, WALL_GRID_IMPRINT};
use crate::wisps::summoning::Summoning;

pub struct MapLoaderPlugin;
impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(MapLoadingStage::LoadMapFile), spawn_load_map_file_stage_tasks)
            .add_systems(OnEnter(MapLoadingStage::DespawnExisting), spawn_despawn_existing_stage_tasks)
            .add_systems(OnEnter(MapLoadingStage::ResetGridsAndResources), spawn_reset_grids_and_resources_stage_tasks)
            .add_systems(OnEnter(MapLoadingStage::ApplyMap), spawn_apply_map_stage_tasks)
            .add_systems(OnEnter(MapLoadingStage::Loaded), |mut next_game_state: ResMut<NextState<GameState>>| { next_game_state.set(GameState::Running); })
            .add_systems(Update, (
                (
                    progress_map_loading_state,
                    DespawnExisistngMapLoadingTask::update.run_if(in_state(MapLoadingStage::DespawnExisting)),
                    ResetGridsAndResourcesMapLoadingTask::update.run_if(in_state(MapLoadingStage::ResetGridsAndResources)),
                    LoadMapFileMapLoadingTask::update.run_if(in_state(MapLoadingStage::LoadMapFile)),
                    ApplyMapMapLoadingTask::update.run_if(in_state(MapLoadingStage::ApplyMap)),
                ).run_if(in_state(GameState::Loading)),
            ))
            .add_observer(LoadMapRequest::on_trigger);
    }
}

#[derive(Event)]
pub struct LoadMapRequest(pub String);
impl LoadMapRequest {
    fn on_trigger(
        trigger: Trigger<LoadMapRequest>,
        mut next_game_state: ResMut<NextState<GameState>>,
        mut next_ui_state: ResMut<NextState<UiInteraction>>,
        mut next_map_loading_stage: ResMut<NextState<MapLoadingStage>>,
        mut map_info: ResMut<MapInfo>,
    ) {
        map_info.name = trigger.event().0.clone(); // Set the map name so the next steps can retrieve it. It desyncs the data, as now the rest of the map info is not yet loaded or contains old data. Not ideal solution.
        next_game_state.set(GameState::Loading);
        next_map_loading_stage.set(MapLoadingStage::Init);
        next_ui_state.set(UiInteraction::Free);
    }
}

/// Check if there are any MapLoadingTasks left. If not, move to the next stage. Repeat until `Loaded` state is reached.
fn progress_map_loading_state(
    stage: ResMut<State<MapLoadingStage>>,
    mut next_stage: ResMut<NextState<MapLoadingStage>>,
    tasks: Query<(), With<MapLoadingTask>>,
) {
    if !tasks.is_empty() { return; }
    next_stage.set(stage.get().next());
}

fn spawn_load_map_file_stage_tasks(mut commands: Commands) { commands.spawn(LoadMapFileMapLoadingTask); }
fn spawn_despawn_existing_stage_tasks(mut commands: Commands) { commands.spawn(DespawnExisistngMapLoadingTask); }
fn spawn_reset_grids_and_resources_stage_tasks(mut commands: Commands) { commands.spawn(ResetGridsAndResourcesMapLoadingTask); }
fn spawn_apply_map_stage_tasks(mut commands: Commands) { commands.spawn(ApplyMapMapLoadingTask); }

#[derive(Component)]
#[require(MapLoadingTask)]
struct DespawnExisistngMapLoadingTask;
impl DespawnExisistngMapLoadingTask {
    fn update(
        mut commands: Commands,
        task: Query<Entity, With<DespawnExisistngMapLoadingTask>>,
        map_bound_entities: Query<Entity, With<MapBound>>,
    ) {
        let Ok(task_entity) =  task.single() else {return; };
        map_bound_entities.iter().for_each(|entity| commands.entity(entity).despawn());
        commands.entity(task_entity).despawn();
    }
}
#[derive(Component)]
#[require(MapLoadingTask)]
struct ResetGridsAndResourcesMapLoadingTask;
impl ResetGridsAndResourcesMapLoadingTask {
    fn update(
        mut commands: Commands,
        map_info: Res<MapInfo>,
        mut obstacles_grid: ResMut<ObstacleGrid>,
        mut emissions_grid: ResMut<EmissionsGrid>,
        mut energy_supply_grid: ResMut<EnergySupplyGrid>,
        mut wisps_grid: ResMut<WispsGrid>,
        mut tower_ranges_grid: ResMut<TowerRangesGrid>,
        tasks: Query<Entity, With<ResetGridsAndResourcesMapLoadingTask>>,
    ) {
        let Ok(task_entity) = tasks.single() else {return; };
        obstacles_grid.resize_and_reset(map_info.bounds());
        emissions_grid.resize_and_reset(map_info.bounds());
        energy_supply_grid.resize_and_reset(map_info.bounds());
        wisps_grid.resize_and_reset(map_info.bounds());
        tower_ranges_grid.resize_and_reset(map_info.bounds());
        commands.entity(task_entity).despawn();
    }
}
#[derive(Component)]
#[require(MapLoadingTask)]
struct LoadMapFileMapLoadingTask;
impl LoadMapFileMapLoadingTask {
    fn update(
        mut commands: Commands,
        mut map_info: ResMut<MapInfo>,
        tasks: Query<Entity, With<LoadMapFileMapLoadingTask>>,
    ) {
        let Ok(task_entity) = tasks.single() else {return; };
        let map_file = MapFile::load(&map_info.name);
        map_info.set(map_file);
        commands.entity(task_entity).despawn();
    }
}

#[derive(Component)]
#[require(MapLoadingTask)]
struct ApplyMapMapLoadingTask;
impl ApplyMapMapLoadingTask {
    fn update(
        mut commands: Commands,
        mut map_info: ResMut<MapInfo>,
        task: Query<Entity, With<ApplyMapMapLoadingTask>>,
        mut obstacles_grid: ResMut<ObstacleGrid>,
        almanach: Res<Almanach>,
    ) {
        let Ok(task_entity) = task.single() else { return; };
        map_info.map_file.apply(&mut commands, &mut obstacles_grid, &almanach);
        commands.entity(task_entity).despawn();
    }
}

/// Represents yaml content for a map
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct MapFile {
    pub width: i32,
    pub height: i32,
    pub buildings: Vec<MapBuilding>,
    pub walls: Vec<GridCoords>,
    pub dark_ores: Vec<GridCoords>,
    pub quantum_fields: Vec<MapQuantumField>,
    #[serde(skip_serializing)] pub objectives: Vec<ObjectiveDetails>,
    #[serde(skip_serializing)] pub summonings: Vec<Summoning>,
}
impl MapFile {
    /// Load a map from a yaml file in the maps directory into a Map struct.
    fn load(map_name: &str) -> Self {
        serde_yaml::from_reader(File::open(format!("maps/{}.yaml", map_name)).unwrap()).unwrap()
    }
    /// Apply the map to the scene.
    fn apply(
        &mut self,
        commands: &mut Commands,
        obstacle_grid: &mut ObstacleGrid,
        almanach: &Almanach,
    ) {
        self.walls.iter().for_each(|wall_coords| {
            let wall_entity = commands.spawn(BuilderWall::new(*wall_coords)).id();
            obstacle_grid.imprint(*wall_coords, Field::Wall(wall_entity), WALL_GRID_IMPRINT);
        });
        let _dark_ores = self.dark_ores.iter().map(|dark_ore_coords| {
            let dark_ore_entity = commands.spawn(BuilderDarkOre::new(*dark_ore_coords)).id();
            obstacle_grid.imprint(*dark_ore_coords, Field::DarkOre(dark_ore_entity), DARK_ORE_GRID_IMPRINT);
            (*dark_ore_coords, dark_ore_entity)
        }).collect::<HashMap<_,_>>();
        self.buildings.iter().for_each(|building| {
            let building_entity = match building.building_type {
                BuildingType::MainBase => commands.spawn(BuilderMainBase::new(building.coords)).id(),
                BuildingType::EnergyRelay => commands.spawn(BuilderEnergyRelay::new(building.coords)).id(),
                BuildingType::ExplorationCenter => commands.spawn(BuilderExplorationCenter::new(building.coords)).id(),
                BuildingType::Tower(tower_type) => {
                    match tower_type {
                        TowerType::Blaster => commands.spawn(BuilderTowerBlaster::new(building.coords)).id(),
                        TowerType::Cannon => commands.spawn(BuilderTowerCannon::new(building.coords)).id(),
                        TowerType::RocketLauncher => commands.spawn(BuilderTowerRocketLauncher::new(building.coords)).id(),
                        TowerType::Emitter => commands.spawn(BuilderTowerEmitter::new(building.coords)).id(),
                    }
                }
                BuildingType::MiningComplex => {
                    // TODO: This won't work as MiningComplex needs special place(type) on obstacle grid, see placing code
                    panic!("Not implemented, read the comment");
                    // let entity = commands.spawn_empty().id();
                    // commands.queue(BuilderMiningComplex::new(entity, building.coords));
                    // entity
                }
            };
            obstacle_grid.imprint(building.coords, Field::Building(building_entity, building.building_type, default()), almanach.get_building_info(building.building_type).grid_imprint);
        });
        self.quantum_fields.iter().for_each(|quantum_field| {
            let grid_imprint = GridImprint::Rectangle { width: quantum_field.size, height: quantum_field.size };
            let quantum_field_entity = commands.spawn(BuilderQuantumField::new(quantum_field.coords, grid_imprint)).id();
            obstacle_grid.imprint(quantum_field.coords, Field::QuantumField(quantum_field_entity), grid_imprint);
        });
        self.objectives.iter().cloned().for_each(|objective_details| {
            commands.spawn(objective_details);
        });
        self.summonings.iter().cloned().for_each(|summoning| {
            commands.spawn(summoning);
        });
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MapBuilding {
    pub building_type: BuildingType,
    pub coords: GridCoords,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MapQuantumField {
    pub coords: GridCoords,
    pub size: i32,
}