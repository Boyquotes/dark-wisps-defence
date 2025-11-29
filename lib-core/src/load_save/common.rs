use bevy::ecs::system::ScheduleSystem;

use crate::lib_prelude::*;

thread_local! {
    static DB_CONNECTION: std::cell::RefCell<Option<(String, rusqlite::Connection)>> = std::cell::RefCell::new(None);
}

pub mod db_migrations {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub trait GameDbHelpers {
    fn register_entity(&self, entity_id: i64) -> rusqlite::Result<usize>;
    fn save_grid_coords(&self, entity_id: i64, pos: GridCoords) -> rusqlite::Result<usize>;
    fn save_marker(&self, table_name: &str, entity_id: i64) -> rusqlite::Result<usize>;
}
impl GameDbHelpers for rusqlite::Connection {
    fn register_entity(&self, entity_id: i64) -> rusqlite::Result<usize> {
        self.execute(
            "INSERT OR IGNORE INTO entities (id) VALUES (?1)",
            [entity_id],
        )
    }

    fn save_grid_coords(&self, entity_id: i64, pos: GridCoords) -> rusqlite::Result<usize> {
        self.execute(
            "INSERT INTO grid_coords (entity_id, x, y) VALUES (?1, ?2, ?3)",
            (entity_id, pos.x, pos.y),
        )
    }

    /// Save entity of the object in its dedicated table. Calls register_entity()
    fn save_marker(&self, table_name: &str, entity_id: i64) -> rusqlite::Result<usize> {
        self.register_entity(entity_id)?;
        let query = format!("INSERT OR REPLACE INTO {} (id) VALUES (?1)", table_name);
        self.execute(&query, [entity_id])
    }
}

/// Helper to access a thread-local database connection.
/// It opens the connection if it's not open or if the path has changed.
pub fn with_db_connection<F>(path: &str, f: F) -> Result<(), Box<dyn std::error::Error>> 
where
    F: FnOnce(&mut rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>>,
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
        if let Some((_, ref mut conn)) = *current {
            f(conn)
        } else {
            unreachable!("Connection should be open")
        }
    })
}

pub trait AppGameLoadSaveExtension {
    fn register_db_loader<T: Loadable>(&mut self, stage: MapLoadingStage2) -> &mut Self;
    fn register_db_saver<M>(&mut self, save_system: impl IntoScheduleConfigs<ScheduleSystem, M>) -> &mut Self;
}
impl AppGameLoadSaveExtension for App {
    fn register_db_loader<T: Loadable>(&mut self, stage: MapLoadingStage2) -> &mut Self {
        if !self.world().contains_resource::<GameLoadRegistry>() {
            self.init_resource::<GameLoadRegistry>();
        }
        let mut registry = self.world_mut().resource_mut::<GameLoadRegistry>();
        registry.register::<T>(stage);

        self
    }
    fn register_db_saver<M>(&mut self, save_system: impl IntoScheduleConfigs<ScheduleSystem, M>) -> &mut Self {
        self.add_systems(PostUpdate, save_system.run_if(on_message::<SaveGameSignal>));
        
        self
    }
}