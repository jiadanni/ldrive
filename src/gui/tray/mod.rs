/// System tray integration for DriveSync
///
/// Provides:
/// - Status icon in system tray
/// - Context menu with quick actions
/// - Status notifications
/// - Desktop notifications for sync events

pub mod tray;
pub mod notifications;

pub use tray::{TrayIcon, SyncStatus};
pub use notifications::NotificationManager;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrayError {
    #[error("Failed to create tray icon: {0}")]
    CreationError(String),

    #[error("Failed to update tray: {0}")]
    UpdateError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),
}

pub type Result<T> = std::result::Result<T, TrayError>;
