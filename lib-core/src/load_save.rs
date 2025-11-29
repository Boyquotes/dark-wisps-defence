use bevy::input::common_conditions::input_just_released;

pub use rusqlite; // Export rusqlite for other crates

use crate::lib_prelude::*;

pub mod db_migrations {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub mod load_save_prelude {
    pub use super::*;
}

pub struct MapSavePlugin;
impl Plugin for MapSavePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameSaveExecutor>()
            .init_resource::<DbEntityMap>()
            .init_resource::<GameLoadRegistry>()
            .add_message::<SaveGameSignal>()
            .add_systems(Update, (
                SaveGameSignal::emit.run_if(input_just_released(KeyCode::KeyZ)),
                LoadGameSignal::emit.run_if(input_just_released(KeyCode::KeyA)),
                process_loading_tasks_system,
            ))
            .add_systems(Last, (
                GameSaveExecutor::on_game_save.run_if(on_message::<SaveGameSignal>),
            ))
            ;
    }
}

/// --- SAVE --- ///

#[derive(Message)]
pub struct SaveGameSignal;
impl SaveGameSignal {
    fn emit(
        mut save_executor: ResMut<GameSaveExecutor>,
        mut writer: MessageWriter<SaveGameSignal>,
    ) {
        save_executor.save_name = "test_save.dwd".into();
        save_executor.objects_to_save.clear();
        writer.write(SaveGameSignal);
    }
}

pub trait Saveable: SSS {
    fn save(self, tx: &rusqlite::Transaction) -> rusqlite::Result<()>;
}
pub struct SaveableBatchCommand<T: Saveable> {
    data: Vec<T>,
}
impl<T: Saveable> FromIterator<T> for SaveableBatchCommand<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut result = Self { data: Vec::new() };
        result.data.extend(iter);
        result
    }
}
impl<T: Saveable> SSS for SaveableBatchCommand<T> {}
impl<T: Saveable> SaveableBatch for SaveableBatchCommand<T> {
    fn save_batch(self: Box<Self>, tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
        for item in self.data {
            item.save(tx)?;
        }
        Ok(())
    }
}
impl<T: Saveable> Command for SaveableBatchCommand<T> {
    fn apply(self, world: &mut World) {
        let mut buffer = world.resource_mut::<GameSaveExecutor>();
        println!("Added {} objects to GameSaveExecutor", self.data.len());
        // Box since we store dyn in GameSaveExecutor
        buffer.objects_to_save.push(Box::new(self));
    }
}
pub trait SaveableBatch: SSS {
    fn save_batch(self: Box<Self>, tx: &rusqlite::Transaction) -> rusqlite::Result<()>;
}

#[derive(Resource, Default)]
pub struct GameSaveExecutor {
    pub save_name: String,
    pub objects_to_save: Vec<Box<dyn SaveableBatch>>,
}
impl GameSaveExecutor {
    fn on_game_save(
        mut save_executor: ResMut<GameSaveExecutor>,
    ) {
        if save_executor.objects_to_save.is_empty() {
            return;
        }
        
        let objects = std::mem::take(&mut save_executor.objects_to_save);
        let save_name = save_executor.save_name.clone();

        std::thread::spawn(move || {
            fn save_process(save_name: &str, objects: Vec<Box<dyn SaveableBatch>>) -> Result<(), Box<dyn std::error::Error>> {
                if std::path::Path::new(save_name).exists() {
                    println!("Removing existing save file '{}'", save_name);
                    std::fs::remove_file(save_name)?;
                }
                // Open the database
                let mut conn = rusqlite::Connection::open(save_name)?;

                // Run migrations
                db_migrations::migrations::runner().run(&mut conn)?;

                // Start transaction
                let tx = conn.transaction()?;

                // Save all objects
                for batch in objects {
                    batch.save_batch(&tx)?;
                }

                // Commit transaction
                tx.commit()?;
                Ok(())
            }

            if let Err(e) = save_process(&save_name, objects) {
                eprintln!("Failed to save game: {}", e);
            } else {
                println!("Game saved successfully to '{}'", save_name);
            }
        });
    }
}

/// --- LOAD --- ///

pub struct MapLoadPlugin;
impl Plugin for MapLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(MapLoadingStage2::LoadMapInfo), spawn_loading_tasks)
            //.add_systems(OnEnter(MapLoadingStage2::LoadResources), spawn_reset_grids_and_resources_stage_tasks)
            .add_systems(OnEnter(MapLoadingStage2::SpawnMapElements), spawn_loading_tasks)
            .add_systems(OnEnter(MapLoadingStage2::Ready), |mut commands: Commands, mut next_game_state: ResMut<NextState<GameState>>| { 
                commands.trigger(DynamicGameEvent::game_started());
                next_game_state.set(GameState::Running); 
            })
            .add_systems(Update, (
                progress_map_loading_state.run_if(in_state(GameState::Loading2)),
            ))
            .add_observer(LoadGameSignal::on_trigger)
            .register_db_loader::<PopulateDbEntityMapTask>(MapLoadingStage2::LoadMapInfo);
    }
}


#[derive(Event)]
pub struct LoadGameSignal(pub String);
impl LoadGameSignal {
    fn emit(
        mut commands: Commands,
    ) {
        println!("Loading game");
        commands.trigger(LoadGameSignal("test_save.dwd".into()));
    }
    fn on_trigger(
        trigger: On<LoadGameSignal>,
        mut commands: Commands,
        mut next_game_state: ResMut<NextState<GameState>>,
        mut next_ui_state: ResMut<NextState<UiInteraction>>,
        mut next_map_loading_stage: ResMut<NextState<MapLoadingStage2>>,
        mut save_executor: ResMut<GameSaveExecutor>,
        map_bound_entities: Query<Entity, With<MapBound>>,
    ) {
        save_executor.save_name = trigger.event().0.clone();
        
        next_game_state.set(GameState::Loading2);
        next_map_loading_stage.set(MapLoadingStage2::Init); // Transition to LoadMapInfo immediately via next()
        next_ui_state.set(UiInteraction::Free);

        // Despawn all existing map elements
        map_bound_entities.iter().for_each(|entity| commands.entity(entity).despawn());
    }
}


thread_local! {
    static DB_CONNECTION: std::cell::RefCell<Option<(String, rusqlite::Connection)>> = std::cell::RefCell::new(None);
}

/// Helper to access a thread-local database connection.
/// It opens the connection if it's not open or if the path has changed.
pub fn with_db_connection<F, R>(path: &str, f: F) -> rusqlite::Result<R>
where
    F: FnOnce(&rusqlite::Connection) -> rusqlite::Result<R>,
{
    DB_CONNECTION.with(|cell| {
        let mut current = cell.borrow_mut();
        
        // Check if we need to open a new connection
        let needs_open = match *current {
            Some((ref p, _)) => p != path,
            None => true,
        };

        if needs_open {
            let conn = rusqlite::Connection::open(path)?;
            *current = Some((path.to_string(), conn));
        }

        // Now we are sure we have a connection
        if let Some((_, ref conn)) = *current {
            f(conn)
        } else {
            unreachable!("Connection should be open")
        }
    })
}

#[derive(Resource, Default)]
pub struct DbEntityMap {
    pub map: HashMap<u64, Entity>,
}

pub enum LoadResult {
    Progressed(usize),
    Finished,
}

pub struct LoadContext<'a, 'w, 's> {
    pub conn: &'a rusqlite::Connection,
    pub commands: &'a mut Commands<'w, 's>,
    pub entity_map: &'a DbEntityMap,
    pub offset: usize,
}
impl<'a, 'w, 's> LoadContext<'a, 'w, 's> {
    pub fn get_entity(&self, old_id: u64) -> Option<Entity> {
        self.entity_map.map.get(&old_id).copied()
    }
}

pub trait Loadable {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult>;
}

pub type LoaderFn = fn(&mut LoadContext) -> rusqlite::Result<LoadResult>;

#[derive(Resource, Default)]
pub struct GameLoadRegistry {
    pub loaders: HashMap<MapLoadingStage2, Vec<LoaderFn>>,
}

impl GameLoadRegistry {
    pub fn register<T: Loadable>(&mut self, phase: MapLoadingStage2) {
        self.loaders.entry(phase).or_default().push(T::load);
    }
}

pub trait AppGameLoadExtension {
    fn register_db_loader<T: Loadable>(&mut self, phase: MapLoadingStage2) -> &mut Self;
}

impl AppGameLoadExtension for App {
    fn register_db_loader<T: Loadable>(&mut self, phase: MapLoadingStage2) -> &mut Self {
        if !self.world().contains_resource::<GameLoadRegistry>() {
            self.init_resource::<GameLoadRegistry>();
        }
        let mut registry = self.world_mut().resource_mut::<GameLoadRegistry>();
        registry.register::<T>(phase);
        self
    }
}

#[derive(Component, Clone)]
pub struct DbLoadingTask {
    pub loader: LoaderFn,
    pub offset: usize,
}

struct PopulateDbEntityMapTask;
impl Loadable for PopulateDbEntityMapTask {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult> {
        let mut map = HashMap::new();
        let mut stmt = ctx.conn.prepare("SELECT id FROM entities")?;
        let rows = stmt.query_map([], |row| row.get::<_, u64>(0))?;
        
        let mut count = 0;
        for row in rows {
            let old_id = row?;
            let new_entity = ctx.commands.spawn_empty().id();
            map.insert(old_id, new_entity);
            count += 1;
        }
        
        println!("Populated EntityMap with {} entities", count);
        ctx.commands.insert_resource(DbEntityMap { map });
        Ok(LoadResult::Finished)
    }
}

pub fn process_loading_tasks_system(
    par_commands: ParallelCommands,
    save_executor: Res<GameSaveExecutor>,
    entity_map: Res<DbEntityMap>,
    mut tasks: Query<(Entity, &mut DbLoadingTask)>,
) {
    let start_time = std::time::Instant::now();
    let time_budget = std::time::Duration::from_millis(5);

    tasks.par_iter_mut().for_each(|(entity, mut task)| {
        par_commands.command_scope(|mut cmd| {
             let _ = with_db_connection(&save_executor.save_name, |conn| {
                 loop {
                     // Check global system budget
                     if start_time.elapsed() > time_budget {
                         break;
                     }

                     let mut ctx = LoadContext {
                         conn,
                         commands: &mut cmd,
                         entity_map: &entity_map,
                         offset: task.offset,
                     };
                     
                     match (task.loader)(&mut ctx) {
                         Ok(LoadResult::Finished) => {
                             cmd.entity(entity).despawn();
                             break; // Task done
                         },
                         Ok(LoadResult::Progressed(count)) => {
                             task.offset += count;
                             // Continue loop to process more if time permits
                         },
                         Err(e) => {
                             eprintln!("Loading task failed: {}", e);
                             cmd.entity(entity).despawn(); // Stop on error
                             break;
                         }
                     }
                 }
                 Ok(())
             });
        });
    });
}

/// Check if there are any MapLoadingTasks (local) or LoadingTask (DB) left.
fn progress_map_loading_state(
    stage: ResMut<State<MapLoadingStage2>>,
    mut next_stage: ResMut<NextState<MapLoadingStage2>>,
    loading_tasks: Query<(), With<DbLoadingTask>>,
) {
    if !loading_tasks.is_empty() { return; }
    next_stage.set(stage.get().next());
}

fn spawn_loading_tasks(
    mut commands: Commands,
    registry: Res<GameLoadRegistry>,
    stage: ResMut<State<MapLoadingStage2>>,
) {
    println!("Spawning loading tasks for phase: {:?}", stage.get());
    let target_phase = stage.get();
    if let Some(loaders) = registry.loaders.get(target_phase) {
        for loader in loaders {
            commands.spawn(DbLoadingTask {
                loader: *loader,
                offset: 0,
            });
        }
    }
}