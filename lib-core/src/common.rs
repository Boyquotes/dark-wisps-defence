use bevy::input::common_conditions::input_just_released;
use rusqlite::Connection;
pub use rusqlite; // Export rusqlite for other crates

use crate::lib_prelude::*;

pub mod db_migrations {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub mod common_prelude {
    pub use super::*;
}

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameSaveExecutor>()
            .init_resource::<EntityMap>()
            .init_resource::<GameLoadRegistry>()
            .add_message::<SaveGameSignal>()
            .add_systems(Update, (
                ColorPulsation::pulsate_sprites_system,
                SaveGameSignal::emit.run_if(input_just_released(KeyCode::KeyZ)),
                process_loading_tasks_system,
            ))
            .add_systems(Last, (
                GameSaveExecutor::on_game_save.run_if(on_message::<SaveGameSignal>),
            ))
            .add_observer(ZDepth::on_insert)
            .add_observer(MaxHealth::on_insert)
            .add_observer(ColorPulsation::on_remove)
            ;
    }
}

pub trait SSS: Send + Sync + 'static {}

// Simple property trait for single value objects. Useful in generic contexts.
pub trait Property {
    fn get(&self) -> f32;
    fn set(&mut self, value: f32);
    fn new(value: f32) -> Self;
}

// Event that carries yaml-define or constant events 
#[derive(Event)]
pub struct DynamicGameEvent(pub String);
impl DynamicGameEvent {
    pub fn game_started() -> Self { DynamicGameEvent("game-started".to_string()) }
}

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
#[derive(Message)]
pub struct LoadGame(pub String);
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
use std::cell::RefCell;

#[derive(Resource, Default)]
pub struct GameSaveExecutor {
    pub save_name: String,
    pub objects_to_save: Vec<Box<dyn SaveableBatch>>,
}

thread_local! {
    static DB_CONNECTION: RefCell<Option<(String, Connection)>> = RefCell::new(None);
}

/// Helper to access a thread-local database connection.
/// It opens the connection if it's not open or if the path has changed.
pub fn with_db_connection<F, R>(path: &str, f: F) -> rusqlite::Result<R>
where
    F: FnOnce(&Connection) -> rusqlite::Result<R>,
{
    DB_CONNECTION.with(|cell| {
        let mut current = cell.borrow_mut();
        
        // Check if we need to open a new connection
        let needs_open = match *current {
            Some((ref p, _)) => p != path,
            None => true,
        };

        if needs_open {
            let conn = Connection::open(path)?;
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

use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct EntityMap {
    pub map: HashMap<u64, Entity>,
}

pub enum LoadResult {
    Progressed(usize),
    Finished,
}

pub struct LoadContext<'a, 'w, 's> {
    pub conn: &'a rusqlite::Connection,
    pub commands: &'a mut Commands<'w, 's>,
    pub entity_map: &'a EntityMap,
    pub offset: usize,
}

impl<'a, 'w, 's> LoadContext<'a, 'w, 's> {
    pub fn get_entity(&self, old_id: u64) -> Option<Entity> {
        self.entity_map.map.get(&old_id).copied()
    }
}

pub trait Loadable: Sized + Component {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult>;
}

pub type LoadFn = fn(&mut LoadContext) -> rusqlite::Result<LoadResult>;

#[derive(Resource, Default)]
pub struct GameLoadRegistry {
    pub loaders: Vec<LoadFn>,
}

impl GameLoadRegistry {
    pub fn register<T: Loadable>(&mut self) {
        self.loaders.push(T::load);
    }
}

#[derive(Component, Clone)]
pub struct LoadingTask {
    pub loader: LoadFn,
    pub offset: usize,
}

/// System to populate EntityMap from the 'entities' table.
/// This should be run once before any component loading systems.
pub fn populate_entity_map_system(
    mut commands: Commands,
    save_executor: Res<GameSaveExecutor>,
    mut entity_map: ResMut<EntityMap>,
) {
    // Simple guard: don't run if already populated
    if !entity_map.map.is_empty() { return; }

    let result = with_db_connection(&save_executor.save_name, |conn| {
        let mut stmt = conn.prepare("SELECT id FROM entities")?;
        let ids_iter = stmt.query_map([], |row| row.get::<_, u64>(0))?;
        
        let mut count = 0;
        for id_result in ids_iter {
            let old_id = id_result?;
            let new_entity = commands.spawn_empty().id();
            entity_map.map.insert(old_id, new_entity);
            count += 1;
        }
        Ok(count)
    });

    match result {
        Ok(count) => println!("Populated EntityMap with {} entities", count),
        Err(e) => eprintln!("Failed to populate EntityMap: {}", e),
    }
}

pub fn start_loading_system(
    mut commands: Commands,
    registry: Res<GameLoadRegistry>,
) {
    for loader in &registry.loaders {
        commands.spawn(LoadingTask {
            loader: *loader,
            offset: 0,
        });
    }
}
pub fn process_loading_tasks_system(
    par_commands: ParallelCommands,
    save_executor: Res<GameSaveExecutor>,
    entity_map: Res<EntityMap>,
    mut tasks: Query<(Entity, &mut LoadingTask)>,
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
                let mut conn = Connection::open(save_name)?;

                // Run migrations
                crate::common::db_migrations::migrations::runner().run(&mut conn)?;

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


// Component for entities that are bound to the map and shall be removed on its change
#[derive(Component, Default)]
pub struct MapBound; 

// Marker for tasks to be performed during MapLoading. 
#[derive(Component, Default)]
pub struct MapLoadingTask;

#[derive(Component, Default)]
pub struct Health {
    current: f32,
    max: f32, // A helper, source of truth is in MaxHealth component
}
impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    pub fn get_current(&self) -> f32 {
        self.current
    }
    pub fn get_max(&self) -> f32 {
        self.max
    }
    pub fn get_percent(&self) -> f32 {
        self.current / self.max
    }
    pub fn decrease(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.);
    }
    pub fn is_dead(&self) -> bool {
        self.current <= 0.
    }
}

#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
#[require(Health)]
pub struct MaxHealth(pub f32);
impl MaxHealth {
    fn on_insert(
        trigger: On<Insert, MaxHealth>,
        mut healths: Query<(&mut Health, &MaxHealth)>,
    ) {
        let Ok((mut health, max_health)) = healths.get_mut(trigger.entity) else { return; };
        if health.current == 0. {
            health.current = max_health.0;
        }
        health.max = max_health.0;
    }
}
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct MovementSpeed(pub f32);
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct AttackSpeed(pub f32);
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct AttackDamage(pub f32);
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct AttackRange(pub f32);
#[derive(Component, Default, Clone, Copy, Property)]
#[component(immutable)]
pub struct EnergySupplyRange(pub f32);


#[derive(Component, Default)]
pub struct ColorPulsation {
    min_brightness: f32,
    max_brightness: f32,
    duration: f32,
    is_increasing: bool,
    delta_change: f32,
}
impl ColorPulsation {
    pub fn new(min_brightness: f32, max_brightness: f32, duration: f32) -> Self {
        let mut color_pulsation = ColorPulsation::default();
        color_pulsation.update_parameters(min_brightness, max_brightness, duration);
        color_pulsation
    }
    pub fn update_parameters(&mut self, min_brightness: f32, max_brightness: f32, duration: f32) {
        assert!(min_brightness < max_brightness, "min_brightness must be less than max_brightness");
        self.min_brightness = min_brightness;
        self.max_brightness = max_brightness;
        self.duration = duration;
        self.delta_change = (max_brightness - min_brightness) / duration;
    }

    fn on_remove(
        trigger: On<Remove, ColorPulsation>,
        mut sprites: Query<&mut Sprite>,
    ) {
        let entity = trigger.entity;
        let Ok(mut sprite) = sprites.get_mut(entity) else { return; };
        match &mut sprite.color {
            Color::Hsla(Hsla {lightness, .. }) => {
                *lightness = 1.0;
            }
            _ => {}
        }
    }

    fn pulsate_sprites_system(
        time: Res<Time>,
        mut sprites: Query<(&mut Sprite, &mut ColorPulsation)>,
    ) {
        for (mut sprite, mut color_pulsation) in sprites.iter_mut() {
            let delta_time = time.delta_secs();
            match &mut sprite.color {
                Color::Hsla(Hsla {lightness, .. }) => {
                    if color_pulsation.is_increasing && *lightness > color_pulsation.max_brightness {
                        color_pulsation.is_increasing = false;
                    } else if !color_pulsation.is_increasing && *lightness < color_pulsation.min_brightness {
                        color_pulsation.is_increasing = true;
                    }
                    *lightness += delta_time * color_pulsation.delta_change * if color_pulsation.is_increasing { 1. } else { -1. }
                }
                _ => {}
            }
        }
    }
}

#[derive(Component)]
#[component(immutable)]
#[require(Transform)]
pub struct ZDepth(pub f32);
impl ZDepth {
    fn on_insert(
        trigger: On<Insert, ZDepth>,
        mut transforms: Query<(&mut Transform, &ZDepth)>,
    ) {
        let entity = trigger.entity;
        let Ok((mut transform, z_depth)) = transforms.get_mut(entity) else { return; };
        transform.translation.z = z_depth.0;
    }
}
impl From<f32> for ZDepth {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

macro_rules! define_z_indexes {
    // Internal macro to handle incrementing the counter
    (@internal $counter:expr, $name:ident) => {
        pub const $name: f32 = $counter;
    };
    (@internal $counter:expr, $name:ident, $($rest:ident),+) => {
        pub const $name: f32 = $counter;
        define_z_indexes!(@internal $counter + 0.001, $($rest),+);
    };
    // Public-facing macro interface
    ($($name:ident),+) => {
        define_z_indexes!(@internal 0.001, $($name),+);
    };
}

define_z_indexes!(
    Z_OBSTACLE,
    Z_OVERLAY_ENERGY_SUPPLY,
    Z_BUILDING,
    Z_WISP,
    Z_GROUND_EFFECT,
    Z_TOWER_TOP,
    Z_MAP_UI,
    Z_AERIAL_UNIT,
    Z_PROJECTILE_UNDER,
    Z_PROJECTILE
);