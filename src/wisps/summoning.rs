use lib_grid::grids::obstacles::ObstacleGrid;

use crate::prelude::*;

use super::components::WispType;
use super::spawning::BuilderWisp;

pub struct SummoningPlugin;
impl Plugin for SummoningPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(MapLoadingStage::ResetGridsAndResources), |mut commands: Commands| { commands.insert_resource(SummoningClock::default());})
            .add_systems(Update, tick_active_summoning_system.run_if(in_state(GameState::Running)))
            .add_observer(on_summoning_activation_event)
            ;
    }
}

// --------------- SUMMONING DEFINITIONS (YAML) ---------------

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
#[require(MapBound, SummoningRuntime)]
pub struct Summoning {
    pub id_name: String,
    pub wisp_types: Vec<WispType>,
    pub area: SpawnArea,
    pub tempo: SpawnTempo,
    pub limit_count: Option<i32>,
    pub activation_event: String,
}
impl Summoning {
    fn get_random_wisp_type(&self, rng: &mut nanorand::tls::TlsWyRand) -> WispType {
        self.wisp_types[rng.generate_range(0..self.wisp_types.len())]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpawnArea {
    Coords { coords: Vec<GridCoords> },
    Rect { origin: GridCoords, width: i32, height: i32 },
    Edge { side: EdgeSide },
    EdgesAll,
}

impl SpawnArea {
    fn get_random_coord(
        &self,
        obstacle_grid: &ObstacleGrid,
        rng: &mut nanorand::tls::TlsWyRand,
    ) -> GridCoords {
        // Nano-rand is off by 1 in i32! 
        match self {
            SpawnArea::Coords { coords } => {
                let idx = rng.generate_range(1..=coords.len());
                coords[idx]
            }
            SpawnArea::Rect { origin, width, height } => {
                let x = origin.x + rng.generate_range(1..=*width);
                let y = origin.y + rng.generate_range(1..=*height);
                GridCoords { x, y }
            }
            SpawnArea::Edge { side } => {
                let (width, height) = obstacle_grid.bounds();
                match side {
                    EdgeSide::Top => GridCoords { x: rng.generate_range(1..=width), y: height - 1 },
                    EdgeSide::Bottom => GridCoords { x: rng.generate_range(1..=width), y: 0 },
                    EdgeSide::Left => GridCoords { x: 0, y: rng.generate_range(1..=height) },
                    EdgeSide::Right => GridCoords { x: width - 1, y: rng.generate_range(1..=height) },
                }
            }
            SpawnArea::EdgesAll => {
                let (width, height) = obstacle_grid.bounds();
                let edge_count = 2 * (width + height - 2);
                let edge_idx = rng.generate_range(1..=edge_count);
                
                if edge_idx < width {
                    GridCoords { x: edge_idx, y: 0 }
                } else if edge_idx < width + height - 1 {
                    GridCoords { x: width - 1, y: edge_idx - width + 1 }
                } else if edge_idx < 2 * width + height - 2 {
                    GridCoords { x: width - 1 - (edge_idx - width - height + 1), y: height - 1 }
                } else {
                    GridCoords { x: 0, y: height - 1 - (edge_idx - 2 * width - height + 2) }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeSide { Top, Bottom, Left, Right }

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpawnTempo {
    /// Spawn `count` wisps every `seconds` (optional jitter). If `count` omitted -> 1.
    Continuous { seconds: f32, #[serde(default)] jitter: f32, #[serde(default = "default_one")] bulk_count: i32 },
}

fn default_one() -> i32 { 1 }

// --------------- SUMMONING ENTITIES AND RUNTIME ---------------
#[derive(Component, Default)]
pub struct SummoningMarkerActive;

#[derive(Component, Default)]
struct SummoningRuntime {
    produced: i32,
    next_spawn_time: f32,
}

#[derive(Resource, Default)]
struct SummoningClock(f32);


fn tick_active_summoning_system(
    time: Res<Time>,
    mut clock: ResMut<SummoningClock>,
    mut commands: Commands,
    obstacle_grid: Res<ObstacleGrid>,
    mut summoning: Query<(&Summoning, &mut SummoningRuntime), With<SummoningMarkerActive>>,
) {
    clock.0 += time.delta_secs();
    let now = clock.0;
    let mut rng = nanorand::tls_rng();

    for (summoning, mut runtime) in &mut summoning {
        // Check if limit is reached(if set)
        let remaining = summoning.limit_count.map(|m| m.saturating_sub(runtime.produced)).unwrap_or(i32::MAX);
        if remaining <= 0 { continue; }

        // Wait until due
        if now < runtime.next_spawn_time { continue; }

        match summoning.tempo {
            SpawnTempo::Continuous { seconds, jitter, bulk_count } => {
                let to_spawn: i32 = std::cmp::min(bulk_count, remaining);
                if to_spawn <= 0 { continue; }
                for _ in 0..(to_spawn as usize) {
                    let grid_coords = summoning.area.get_random_coord(&obstacle_grid, &mut rng);
                    let wisp_type = summoning.get_random_wisp_type(&mut rng);
                    commands.spawn(BuilderWisp::new(wisp_type, grid_coords));
                }
                runtime.produced = runtime.produced.saturating_add(to_spawn);
                let j = if jitter > 0.0 { (rng.generate::<f32>() * 2.0 - 1.0) * jitter } else { 0.0 };
                runtime.next_spawn_time = now + (seconds + j);
            }
        }
    }
}

fn on_summoning_activation_event(
    trigger: On<DynamicGameEvent>,
    mut commands: Commands,
    summonings: Query<(Entity, &Summoning), Without<SummoningMarkerActive>>,
) {
    let event = &trigger.event().0;
    for (entity, summoning) in summonings.iter() {
        if event != &summoning.activation_event { continue; }
        commands.entity(entity).insert(SummoningMarkerActive);
    }
}