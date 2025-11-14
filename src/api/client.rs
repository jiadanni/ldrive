/// Google Drive API client for file operations
///
/// Provides high-level interface for interacting with Google Drive:
/// - List files and folders
/// - Upload and download files
/// - Update file metadata
/// - Delete files
/// - Handle Team Drives

use crate::api::{auth::Credentials, types::*, ApiError, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";
const UPLOAD_API_BASE: &str = "https://www.googleapis.com/upload/drive/v3";

pub struct DriveClient {
    client: Client,
    credentials: Credentials,
}

impl DriveClient {
    pub fn new(credentials: Credentials) -> Self {
        Self {
            client: Client::new(),
            credentials,
        }
    }

    /// Update credentials (for token refresh)
    pub fn update_credentials(&mut self, credentials: Credentials) {
        self.credentials = credentials;
    }

    /// Get current credentials
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    /// List files in a specific folder
    pub async fn list_files(&self, folder_id: Option<&str>, page_size: Option<i32>) -> Result<FileList> {
        let mut url = format!("{}/files", DRIVE_API_BASE);
        let mut query_params = vec![
            ("fields", "nextPageToken,files(id,name,mimeType,size,createdTime,modifiedTime,parents,md5Checksum,webViewLink)".to_string()),
        ];

        if let Some(size) = page_size {
            query_params.push(("pageSize", size.to_string()));
        }

        if let Some(folder) = folder_id {
            query_params.push(("q", format!("'{}' in parents", folder)));
        }

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .bearer_auth(&self.credentials.access_token)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        self.handle_response(response).await
    }

    /// Get file metadata
    pub async fn get_file_metadata(&self, file_id: &str) -> Result<DriveFileMetadata> {
        let url = format!("{}/files/{}", DRIVE_API_BASE, file_id);
        let response = self
            .client
            .get(&url)
            .query(&[(
                "fields",
                "id,name,mimeType,size,createdTime,modifiedTime,parents,md5Checksum,version",
            )])
            .bearer_auth(&self.credentials.access_token)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        self.handle_response(response).await
    }

    /// Download file content
    pub async fn download_file(&self, file_id: &str, destination: &Path) -> Result<()> {
        let url = format!("{}/files/{}?alt=media", DRIVE_API_BASE, file_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.credentials.access_token)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        let mut file = File::create(destination)
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to create file: {}", e)))?;

        file.write_all(&bytes)
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to write file: {}", e)))?;

        tracing::info!("Downloaded file {} to {:?}", file_id, destination);
        Ok(())
    }

    /// Upload file content
    pub async fn upload_file(&self, source: &Path, parent_id: Option<&str>, name: Option<&str>) -> Result<DriveFile> {
        let file_name = name.unwrap_or_else(|| {
            source
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
        });

        // Read file content
        let mut file = File::open(source)
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to open file: {}", e)))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to read file: {}", e)))?;

        // Prepare metadata
        let mut metadata = json!({
            "name": file_name,
        });

        if let Some(parent) = parent_id {
            metadata["parents"] = json!([parent]);
        }

        // Multipart upload
        let url = format!("{}/files?uploadType=multipart", UPLOAD_API_BASE);

        let metadata_part = serde_json::to_vec(&metadata)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to serialize metadata: {}", e)))?;

        let boundary = format!("==============={}==", uuid::Uuid::new_v4().simple());
        let mut body = Vec::new();

        // Metadata part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body.extend_from_slice(&metadata_part);
        body.extend_from_slice(b"\r\n");

        // File content part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(&content);
        body.extend_from_slice(b"\r\n");

        // End boundary
        body.extend_from_slice(format!("--{}--", boundary).as_bytes());

        let response = self
            .client
            .post(&url)
            .header("Content-Type", format!("multipart/related; boundary={}", boundary))
            .bearer_auth(&self.credentials.access_token)
            .body(body)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        self.handle_response(response).await
    }

    /// Update file content
    pub async fn update_file(&self, file_id: &str, source: &Path) -> Result<DriveFile> {
        let mut file = File::open(source)
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to open file: {}", e)))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to read file: {}", e)))?;

        let url = format!("{}/files/{}?uploadType=media", UPLOAD_API_BASE, file_id);

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&self.credentials.access_token)
            .body(content)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        self.handle_response(response).await
    }

    /// Delete file
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        let url = format!("{}/files/{}", DRIVE_API_BASE, file_id);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&self.credentials.access_token)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        if response.status().is_success() || response.status() == StatusCode::NO_CONTENT {
            tracing::info!("Deleted file {}", file_id);
            Ok(())
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    /// Create folder
    pub async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<DriveFile> {
        let url = format!("{}/files", DRIVE_API_BASE);

        let mut metadata = json!({
            "name": name,
            "mimeType": "application/vnd.google-apps.folder",
        });

        if let Some(parent) = parent_id {
            metadata["parents"] = json!([parent]);
        }

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.credentials.access_token)
            .json(&metadata)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        self.handle_response(response).await
    }

    /// Get changes since a specific change token
    pub async fn get_changes(&self, start_token: &str) -> Result<ChangeList> {
        let url = format!("{}/changes", DRIVE_API_BASE);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("pageToken", start_token),
                ("fields", "nextPageToken,newStartPageToken,changes(fileId,removed,file(id,name,mimeType,size,modifiedTime,parents,md5Checksum))"),
            ])
            .bearer_auth(&self.credentials.access_token)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        self.handle_response(response).await
    }

    /// Get the start page token for change tracking
    pub async fn get_start_page_token(&self) -> Result<String> {
        let url = format!("{}/changes/startPageToken", DRIVE_API_BASE);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.credentials.access_token)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        #[derive(Deserialize)]
        struct StartPageToken {
            #[serde(rename = "startPageToken")]
            start_page_token: String,
        }

        let result: StartPageToken = self.handle_response(response).await?;
        Ok(result.start_page_token)
    }

    /// Get information about the user's Drive storage
    pub async fn get_about(&self) -> Result<AboutDrive> {
        let url = format!("{}/about", DRIVE_API_BASE);

        let response = self
            .client
            .get(&url)
            .query(&[("fields", "storageQuota,user")])
            .bearer_auth(&self.credentials.access_token)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        self.handle_response(response).await
    }

    /// Handle successful API responses
    async fn handle_response<T: serde::de::DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    /// Handle API error responses
    async fn handle_error_response(&self, response: reqwest::Response) -> ApiError {
        let status = response.status();

        match status {
            StatusCode::UNAUTHORIZED => ApiError::AuthenticationError("Unauthorized - token may be expired".to_string()),
            StatusCode::FORBIDDEN => ApiError::PermissionDenied("Access forbidden".to_string()),
            StatusCode::NOT_FOUND => ApiError::FileNotFound("Resource not found".to_string()),
            StatusCode::TOO_MANY_REQUESTS => ApiError::RateLimitExceeded,
            _ => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                ApiError::InvalidResponse(format!("API error ({}): {}", status, error_text))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests will be added as implementation progresses
}
