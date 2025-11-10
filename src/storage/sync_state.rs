/// Sync state tracking for files and folders

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub id: String,
    pub name: String,
    pub path: String,
    pub mime_type: String,
    pub size: Option<u64>,
    pub md5_checksum: Option<String>,
    pub modified_time: DateTime<Utc>,
    pub parent_id: Option<String>,
    pub is_folder: bool,
    pub sync_status: SyncStatus,
    pub last_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    Synced,
    Syncing,
    Pending,
    Error,
    Conflict,
    Paused,
}

impl ToString for SyncStatus {
    fn to_string(&self) -> String {
        match self {
            SyncStatus::Synced => "synced",
            SyncStatus::Syncing => "syncing",
            SyncStatus::Pending => "pending",
            SyncStatus::Error => "error",
            SyncStatus::Conflict => "conflict",
            SyncStatus::Paused => "paused",
        }
        .to_string()
    }
}

impl SyncStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "synced" => Some(SyncStatus::Synced),
            "syncing" => Some(SyncStatus::Syncing),
            "pending" => Some(SyncStatus::Pending),
            "error" => Some(SyncStatus::Error),
            "conflict" => Some(SyncStatus::Conflict),
            "paused" => Some(SyncStatus::Paused),
            _ => None,
        }
    }
}

pub struct SyncState;

impl SyncState {
    // TODO: Implement sync state management methods
}
