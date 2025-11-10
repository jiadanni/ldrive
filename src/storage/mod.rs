/// Local storage and database management
///
/// Handles:
/// - SQLite database for sync state
/// - File metadata caching
/// - Sync history tracking
/// - Conflict tracking

pub mod database;
pub mod sync_state;

pub use database::Database;
pub use sync_state::{FileState, SyncState};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("File not found in database: {0}")]
    FileNotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;
