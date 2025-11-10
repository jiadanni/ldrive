/// Google Drive API client module
///
/// This module handles all interactions with the Google Drive API, including:
/// - OAuth 2.0 authentication and token management
/// - File operations (list, upload, download, delete, update)
/// - Metadata retrieval and updates
/// - Push notification subscriptions
/// - Team Drive support

pub mod auth;
pub mod client;
pub mod types;

pub use auth::AuthManager;
pub use client::DriveClient;
pub use types::{DriveFile, DriveFileMetadata, FileChange};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("API rate limit exceeded")]
    RateLimitExceeded,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid response from API: {0}")]
    InvalidResponse(String),

    #[error("Storage quota exceeded")]
    QuotaExceeded,
}

pub type Result<T> = std::result::Result<T, ApiError>;
