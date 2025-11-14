/// Desktop notification management
///
/// Provides desktop notifications for sync events using notify-rust

use super::{Result, TrayError};
use notify_rust::{Notification, Timeout, Urgency};

pub struct NotificationManager {
    app_name: String,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            app_name: "DriveSync".to_string(),
        }
    }

    /// Show a sync completion notification
    pub fn notify_sync_complete(&self, files_synced: usize) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("Sync Complete")
            .body(&format!("Successfully synced {} file(s)", files_synced))
            .icon("emblem-synchronizing-symbolic")
            .timeout(Timeout::Milliseconds(5000))
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show a sync error notification
    pub fn notify_sync_error(&self, error: &str) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("Sync Error")
            .body(error)
            .icon("dialog-error-symbolic")
            .urgency(Urgency::Critical)
            .timeout(Timeout::Never)
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show a file conflict notification
    pub fn notify_conflict(&self, filename: &str) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("File Conflict")
            .body(&format!("Conflict detected: {}", filename))
            .icon("dialog-warning-symbolic")
            .urgency(Urgency::Normal)
            .timeout(Timeout::Never)
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show a storage quota warning
    pub fn notify_quota_warning(&self, percent_used: f64) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("Storage Almost Full")
            .body(&format!(
                "Your Google Drive is {:.0}% full. Consider freeing up space.",
                percent_used
            ))
            .icon("drive-harddisk-symbolic")
            .urgency(Urgency::Normal)
            .timeout(Timeout::Milliseconds(10000))
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show an upload complete notification
    pub fn notify_upload_complete(&self, filename: &str) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("Upload Complete")
            .body(&format!("Uploaded: {}", filename))
            .icon("cloud-symbolic")
            .timeout(Timeout::Milliseconds(3000))
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show a download complete notification
    pub fn notify_download_complete(&self, filename: &str) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("Download Complete")
            .body(&format!("Downloaded: {}", filename))
            .icon("folder-download-symbolic")
            .timeout(Timeout::Milliseconds(3000))
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show a generic notification
    pub fn notify(&self, summary: &str, body: &str, icon: Option<&str>) -> Result<()> {
        let mut notification = Notification::new();
        notification
            .appname(&self.app_name)
            .summary(summary)
            .body(body)
            .timeout(Timeout::Milliseconds(5000));

        if let Some(icon_name) = icon {
            notification.icon(icon_name);
        }

        notification
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show an authentication success notification
    pub fn notify_auth_success(&self, email: &str) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("Authentication Successful")
            .body(&format!("Connected to Google Drive as {}", email))
            .icon("emblem-default-symbolic")
            .timeout(Timeout::Milliseconds(5000))
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show a connection lost notification
    pub fn notify_offline(&self) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("Connection Lost")
            .body("DriveSync is offline. Will resume when connection is restored.")
            .icon("network-offline-symbolic")
            .urgency(Urgency::Normal)
            .timeout(Timeout::Milliseconds(5000))
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Show a connection restored notification
    pub fn notify_online(&self) -> Result<()> {
        Notification::new()
            .appname(&self.app_name)
            .summary("Connection Restored")
            .body("DriveSync is back online. Resuming sync...")
            .icon("network-idle-symbolic")
            .timeout(Timeout::Milliseconds(3000))
            .show()
            .map_err(|e| TrayError::NotificationError(e.to_string()))?;

        Ok(())
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_manager_creation() {
        let manager = NotificationManager::new();
        assert_eq!(manager.app_name, "DriveSync");
    }
}
