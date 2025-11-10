/// Synchronization engine module
///
/// Handles:
/// - Bi-directional sync between local and remote
/// - File change monitoring (inotify)
/// - Conflict detection and resolution
/// - Sync scheduling and prioritization

pub mod engine;
pub mod monitor;
pub mod resolver;

pub use engine::SyncEngine;
pub use monitor::FileMonitor;
pub use resolver::ConflictResolver;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("API error: {0}")]
    ApiError(#[from] crate::api::ApiError),

    #[error("Storage error: {0}")]
    StorageError(#[from] crate::storage::StorageError),

    #[error("File conflict: {0}")]
    Conflict(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Sync failed: {0}")]
    SyncFailed(String),
}

pub type Result<T> = std::result::Result<T, SyncError>;
