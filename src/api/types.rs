/// Type definitions for Google Drive API entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub size: Option<u64>,
    #[serde(rename = "createdTime")]
    pub created_time: DateTime<Utc>,
    #[serde(rename = "modifiedTime")]
    pub modified_time: DateTime<Utc>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(skip)]
    pub is_folder: bool,
    #[serde(rename = "md5Checksum")]
    pub md5_checksum: Option<String>,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFileMetadata {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub size: Option<u64>,
    #[serde(rename = "createdTime")]
    pub created_time: DateTime<Utc>,
    #[serde(rename = "modifiedTime")]
    pub modified_time: DateTime<Utc>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(rename = "md5Checksum")]
    pub md5_checksum: Option<String>,
    #[serde(deserialize_with = "deserialize_version")]
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileList {
    #[serde(default)]
    pub files: Vec<DriveFile>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(default)]
    pub removed: bool,
    pub file: Option<DriveFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeList {
    #[serde(default)]
    pub changes: Vec<Change>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "newStartPageToken")]
    pub new_start_page_token: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AboutDrive {
    #[serde(rename = "storageQuota")]
    pub storage_quota: StorageQuota,
    pub user: Option<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuota {
    #[serde(deserialize_with = "deserialize_u64")]
    pub limit: u64,
    #[serde(deserialize_with = "deserialize_u64")]
    pub usage: u64,
    #[serde(rename = "usageInDrive", deserialize_with = "deserialize_u64")]
    pub usage_in_drive: u64,
    #[serde(rename = "usageInDriveTrash", deserialize_with = "deserialize_optional_u64")]
    pub usage_in_drive_trash: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email_address: String,
    #[serde(rename = "photoLink")]
    pub photo_link: Option<String>,
}

impl DriveFile {
    pub fn is_folder(&self) -> bool {
        self.mime_type == "application/vnd.google-apps.folder"
    }
}

impl Change {
    pub fn to_file_change(&self) -> FileChange {
        let change_type = if self.removed {
            ChangeType::Deleted
        } else if self.file.is_some() {
            ChangeType::Modified
        } else {
            ChangeType::Created
        };

        FileChange {
            file_id: self.file_id.clone(),
            change_type,
            time: self.file.as_ref().map(|f| f.modified_time).unwrap_or_else(Utc::now),
            file: self.file.clone(),
        }
    }
}

// Custom deserializers for handling string numbers from Google API
fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(D::Error::custom)
}

fn deserialize_optional_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let s: Option<String> = Deserialize::deserialize(deserializer)?;
    match s {
        Some(s) => s.parse::<u64>().map(Some).map_err(D::Error::custom),
        None => Ok(None),
    }
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<i64>().map_err(D::Error::custom)
}
