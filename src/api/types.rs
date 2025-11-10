/// Type definitions for Google Drive API entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: Option<u64>,
    pub created_time: DateTime<Utc>,
    pub modified_time: DateTime<Utc>,
    pub parents: Vec<String>,
    pub is_folder: bool,
    pub md5_checksum: Option<String>,
    pub web_view_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFileMetadata {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: Option<u64>,
    pub created_time: DateTime<Utc>,
    pub modified_time: DateTime<Utc>,
    pub parents: Vec<String>,
    pub md5_checksum: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub file_id: String,
    pub change_type: ChangeType,
    pub time: DateTime<Utc>,
    pub file: Option<DriveFile>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Moved,
}

impl DriveFile {
    pub fn is_folder(&self) -> bool {
        self.mime_type == "application/vnd.google-apps.folder"
    }
}
