/// SQLite database for storing sync state and metadata

use crate::storage::{Result, StorageError};
use rusqlite::{Connection, params};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                size INTEGER,
                md5_checksum TEXT,
                modified_time INTEGER NOT NULL,
                parent_id TEXT,
                is_folder INTEGER NOT NULL,
                sync_status TEXT NOT NULL,
                last_sync INTEGER,
                UNIQUE(path)
            );

            CREATE TABLE IF NOT EXISTS sync_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id TEXT NOT NULL,
                action TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                success INTEGER NOT NULL,
                error_message TEXT,
                FOREIGN KEY(file_id) REFERENCES files(id)
            );

            CREATE TABLE IF NOT EXISTS conflicts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id TEXT NOT NULL,
                local_modified INTEGER NOT NULL,
                remote_modified INTEGER NOT NULL,
                resolution TEXT,
                resolved INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(file_id) REFERENCES files(id)
            );

            CREATE INDEX IF NOT EXISTS idx_files_parent ON files(parent_id);
            CREATE INDEX IF NOT EXISTS idx_files_sync_status ON files(sync_status);
            CREATE INDEX IF NOT EXISTS idx_sync_history_file ON sync_history(file_id);
            CREATE INDEX IF NOT EXISTS idx_conflicts_resolved ON conflicts(resolved);
            "#,
        )?;

        Ok(())
    }

    /// Get the underlying connection (for advanced queries)
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path());
        assert!(db.is_ok());
    }
}
