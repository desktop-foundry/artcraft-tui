use artcraft_events::DomainEvent;
use rusqlite::{params, Connection};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("lock poisoned")]
    Lock,
}

pub struct Journal {
    conn: Mutex<Connection>,
}

impl Journal {
    pub fn open(path: &std::path::Path) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, payload TEXT NOT NULL, created_at TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, payload TEXT NOT NULL, created_at TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn record(&self, event: &DomainEvent) -> Result<(), StorageError> {
        let payload = serde_json::to_string(event).unwrap_or_default();
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        conn.execute(
            "INSERT INTO events (name, payload, created_at) VALUES (?1, ?2, datetime('now'))",
            params![event.name(), payload],
        )?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        let c: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))?;
        Ok(c)
    }

    pub fn set_pref(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        conn.execute(
            "INSERT INTO kv (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_pref(&self, key: &str) -> Result<Option<String>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        let mut stmt = conn.prepare("SELECT value FROM kv WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}

impl artcraft_events::EventSink for Journal {
    fn record(&self, event: DomainEvent) {
        let _ = self.record(&event);
    }
}
