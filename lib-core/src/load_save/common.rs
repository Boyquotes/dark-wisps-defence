use crate::lib_prelude::*;

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
