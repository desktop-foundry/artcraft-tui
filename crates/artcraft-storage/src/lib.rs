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

pub struct EventStore {
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

impl EventStore {
    pub fn path() -> std::path::PathBuf {
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| directories::BaseDirs::new().map(|b| b.data_dir().to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        base.join("artcraft-tui")
            .join("artcraft-tui-domain-events.db")
    }

    pub fn open_default() -> Result<Self, StorageError> {
        Self::open(&Self::path())
    }

    pub fn open(path: &std::path::Path) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS event_domains (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_name TEXT NOT NULL,
                payload TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                schema_version TEXT NOT NULL DEFAULT '1'
            );
            CREATE TABLE IF NOT EXISTS pending_work (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                selection TEXT NOT NULL,
                created_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending'
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn record_event(&self, event_name: &str, payload: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        conn.execute(
            "INSERT INTO event_domains (event_name, payload, occurred_at) VALUES (?1, ?2, datetime('now'))",
            params![event_name, payload],
        )?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        let c: i64 = conn.query_row("SELECT COUNT(*) FROM event_domains", [], |r| r.get(0))?;
        Ok(c)
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<(String, String)>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        let mut stmt = conn
            .prepare("SELECT event_name, payload FROM event_domains ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}

impl artcraft_events::EventSink for EventStore {
    fn record(&self, event: artcraft_events::DomainEvent) {
        let payload = serde_json::to_string(&event).unwrap_or_default();
        // JTD validation (best-effort)
        let name = event.name();
        if !event.validate_self() {
            // still record but mark? We'll record anyway for audit.
        }
        let _ = self.record_event(name, &payload);
    }
}

impl EventStore {
    pub fn save_pending(&self, selection: &[String]) -> Result<(), StorageError> {
        let json = serde_json::to_string(selection).unwrap_or_default();
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        // replace any existing pending
        conn.execute("DELETE FROM pending_work WHERE status='pending'", [])?;
        conn.execute(
            "INSERT INTO pending_work (selection, created_at, status) VALUES (?1, datetime('now'), 'pending')",
            params![json],
        )?;
        Ok(())
    }

    pub fn load_pending(&self) -> Result<Vec<String>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        let mut stmt = conn.prepare(
            "SELECT selection FROM pending_work WHERE status='pending' ORDER BY id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let s: String = row.get(0)?;
            Ok(serde_json::from_str(&s).unwrap_or_default())
        } else {
            Ok(vec![])
        }
    }

    pub fn clear_pending(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        conn.execute(
            "UPDATE pending_work SET status='done' WHERE status='pending'",
            [],
        )?;
        Ok(())
    }
}
