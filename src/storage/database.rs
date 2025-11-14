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

    /// Insert or update a file record
    pub fn upsert_file(&self, file: &crate::storage::FileState) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO files (id, name, path, mime_type, size, md5_checksum, modified_time,
                             parent_id, is_folder, sync_status, last_sync)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                path = excluded.path,
                mime_type = excluded.mime_type,
                size = excluded.size,
                md5_checksum = excluded.md5_checksum,
                modified_time = excluded.modified_time,
                parent_id = excluded.parent_id,
                is_folder = excluded.is_folder,
                sync_status = excluded.sync_status,
                last_sync = excluded.last_sync
            "#,
            params![
                file.id,
                file.name,
                file.path,
                file.mime_type,
                file.size,
                file.md5_checksum,
                file.modified_time.timestamp(),
                file.parent_id,
                file.is_folder as i32,
                file.sync_status.to_string(),
                file.last_sync.map(|t| t.timestamp()),
            ],
        )?;
        Ok(())
    }

    /// Get a file by its ID
    pub fn get_file(&self, id: &str) -> Result<Option<crate::storage::FileState>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, mime_type, size, md5_checksum, modified_time,
                    parent_id, is_folder, sync_status, last_sync
             FROM files WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(crate::storage::FileState {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                md5_checksum: row.get(5)?,
                modified_time: chrono::DateTime::from_timestamp(row.get(6)?, 0)
                    .unwrap_or_default(),
                parent_id: row.get(7)?,
                is_folder: row.get::<_, i32>(8)? != 0,
                sync_status: crate::storage::SyncStatus::from_str(&row.get::<_, String>(9)?)
                    .unwrap_or(crate::storage::SyncStatus::Pending),
                last_sync: row.get::<_, Option<i64>>(10)?
                    .and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
            })
        });

        match result {
            Ok(file) => Ok(Some(file)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get a file by its local path
    pub fn get_file_by_path(&self, path: &str) -> Result<Option<crate::storage::FileState>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, mime_type, size, md5_checksum, modified_time,
                    parent_id, is_folder, sync_status, last_sync
             FROM files WHERE path = ?1"
        )?;

        let result = stmt.query_row(params![path], |row| {
            Ok(crate::storage::FileState {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                md5_checksum: row.get(5)?,
                modified_time: chrono::DateTime::from_timestamp(row.get(6)?, 0)
                    .unwrap_or_default(),
                parent_id: row.get(7)?,
                is_folder: row.get::<_, i32>(8)? != 0,
                sync_status: crate::storage::SyncStatus::from_str(&row.get::<_, String>(9)?)
                    .unwrap_or(crate::storage::SyncStatus::Pending),
                last_sync: row.get::<_, Option<i64>>(10)?
                    .and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
            })
        });

        match result {
            Ok(file) => Ok(Some(file)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all files with a specific sync status
    pub fn get_files_by_status(&self, status: crate::storage::SyncStatus) -> Result<Vec<crate::storage::FileState>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, mime_type, size, md5_checksum, modified_time,
                    parent_id, is_folder, sync_status, last_sync
             FROM files WHERE sync_status = ?1"
        )?;

        let files = stmt.query_map(params![status.to_string()], |row| {
            Ok(crate::storage::FileState {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                md5_checksum: row.get(5)?,
                modified_time: chrono::DateTime::from_timestamp(row.get(6)?, 0)
                    .unwrap_or_default(),
                parent_id: row.get(7)?,
                is_folder: row.get::<_, i32>(8)? != 0,
                sync_status: crate::storage::SyncStatus::from_str(&row.get::<_, String>(9)?)
                    .unwrap_or(crate::storage::SyncStatus::Pending),
                last_sync: row.get::<_, Option<i64>>(10)?
                    .and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(files)
    }

    /// Get all files (optionally filtered by parent)
    pub fn get_all_files(&self, parent_id: Option<&str>) -> Result<Vec<crate::storage::FileState>> {
        let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(parent) = parent_id {
            (
                "SELECT id, name, path, mime_type, size, md5_checksum, modified_time,
                        parent_id, is_folder, sync_status, last_sync
                 FROM files WHERE parent_id = ?1".to_string(),
                vec![Box::new(parent.to_string())],
            )
        } else {
            (
                "SELECT id, name, path, mime_type, size, md5_checksum, modified_time,
                        parent_id, is_folder, sync_status, last_sync
                 FROM files".to_string(),
                vec![],
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

        let files = stmt.query_map(&params_refs[..], |row| {
            Ok(crate::storage::FileState {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                md5_checksum: row.get(5)?,
                modified_time: chrono::DateTime::from_timestamp(row.get(6)?, 0)
                    .unwrap_or_default(),
                parent_id: row.get(7)?,
                is_folder: row.get::<_, i32>(8)? != 0,
                sync_status: crate::storage::SyncStatus::from_str(&row.get::<_, String>(9)?)
                    .unwrap_or(crate::storage::SyncStatus::Pending),
                last_sync: row.get::<_, Option<i64>>(10)?
                    .and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(files)
    }

    /// Delete a file record
    pub fn delete_file(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM files WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Update file sync status
    pub fn update_sync_status(&self, id: &str, status: crate::storage::SyncStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE files SET sync_status = ?1, last_sync = ?2 WHERE id = ?3",
            params![status.to_string(), chrono::Utc::now().timestamp(), id],
        )?;
        Ok(())
    }

    /// Add sync history entry
    pub fn add_sync_history(&self, file_id: &str, action: &str, success: bool, error: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sync_history (file_id, action, timestamp, success, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                file_id,
                action,
                chrono::Utc::now().timestamp(),
                success as i32,
                error,
            ],
        )?;
        Ok(())
    }

    /// Add a conflict record
    pub fn add_conflict(
        &self,
        file_id: &str,
        local_modified: chrono::DateTime<chrono::Utc>,
        remote_modified: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO conflicts (file_id, local_modified, remote_modified, resolved)
             VALUES (?1, ?2, ?3, 0)",
            params![
                file_id,
                local_modified.timestamp(),
                remote_modified.timestamp(),
            ],
        )?;
        Ok(())
    }

    /// Get unresolved conflicts
    pub fn get_unresolved_conflicts(&self) -> Result<Vec<ConflictRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, local_modified, remote_modified
             FROM conflicts WHERE resolved = 0"
        )?;

        let conflicts = stmt.query_map([], |row| {
            Ok(ConflictRecord {
                id: row.get(0)?,
                file_id: row.get(1)?,
                local_modified: chrono::DateTime::from_timestamp(row.get(2)?, 0)
                    .unwrap_or_default(),
                remote_modified: chrono::DateTime::from_timestamp(row.get(3)?, 0)
                    .unwrap_or_default(),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(conflicts)
    }

    /// Mark a conflict as resolved
    pub fn resolve_conflict(&self, conflict_id: i64, resolution: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE conflicts SET resolved = 1, resolution = ?1 WHERE id = ?2",
            params![resolution, conflict_id],
        )?;
        Ok(())
    }

    /// Get sync history for a file
    pub fn get_sync_history(&self, file_id: &str, limit: usize) -> Result<Vec<SyncHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, action, timestamp, success, error_message
             FROM sync_history WHERE file_id = ?1
             ORDER BY timestamp DESC LIMIT ?2"
        )?;

        let entries = stmt.query_map(params![file_id, limit], |row| {
            Ok(SyncHistoryEntry {
                id: row.get(0)?,
                file_id: row.get(1)?,
                action: row.get(2)?,
                timestamp: chrono::DateTime::from_timestamp(row.get(3)?, 0)
                    .unwrap_or_default(),
                success: row.get::<_, i32>(4)? != 0,
                error_message: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Get count of files by status
    pub fn count_by_status(&self, status: crate::storage::SyncStatus) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM files WHERE sync_status = ?1",
            params![status.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Clear all file records (for testing/reset)
    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM files", [])?;
        self.conn.execute("DELETE FROM sync_history", [])?;
        self.conn.execute("DELETE FROM conflicts", [])?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ConflictRecord {
    pub id: i64,
    pub file_id: String,
    pub local_modified: chrono::DateTime<chrono::Utc>,
    pub remote_modified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct SyncHistoryEntry {
    pub id: i64,
    pub file_id: String,
    pub action: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub error_message: Option<String>,
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
