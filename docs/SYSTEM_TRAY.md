# System Tray Integration Guide

This guide explains how to use DriveSync's system tray integration for monitoring and controlling sync operations from your desktop.

## Overview

DriveSync integrates with your system tray (notification area) to provide:
- Visual status indicators
- Quick access to common actions
- Desktop notifications for sync events
- At-a-glance storage usage information

## Features

### Status Icons

The tray icon changes based on sync status:

| Icon | Status | Description |
|------|--------|-------------|
| 🔄 | **Syncing** | Files are actively being synchronized |
| ✓ | **Idle/Synced** | All files are up to date |
| ⏸️ | **Paused** | Synchronization is paused |
| ⚠️ | **Error** | An error occurred during sync |
| 📡 | **Offline** | No internet connection |

### Context Menu

Right-click (or left-click, depending on your desktop environment) the tray icon to access:

- **Pause/Resume Sync** - Temporarily stop or restart synchronization
- **Open Sync Folder** - Opens your Google Drive folder in file manager
- **Settings** - Access DriveSync configuration
- **Quit** - Exit DriveSync

### Tooltip Information

Hover over the tray icon to see:
- Current sync status
- Logged-in account email
- Storage usage (e.g., "8.5 GB / 15.0 GB (57%)")

## Desktop Notifications

DriveSync shows notifications for important events:

### Sync Events
- **Sync Complete** - When synchronization finishes successfully
- **Upload Complete** - Individual file upload notifications
- **Download Complete** - Individual file download notifications

### Status Changes
- **Authentication Success** - When you sign in to Google Drive
- **Connection Lost** - When internet connection is unavailable
- **Connection Restored** - When connection comes back online

### Issues and Warnings
- **Sync Error** - When a sync operation fails
- **File Conflict** - When the same file is modified locally and remotely
- **Storage Almost Full** - When you're running low on Drive storage (>90%)

## Desktop Environment Support

DriveSync uses the **StatusNotifierItem** protocol which is supported by:

### Full Support
- ✅ **KDE Plasma** 5.0+
- ✅ **GNOME** 3.26+ (with AppIndicator extension)
- ✅ **Cinnamon** 4.0+
- ✅ **MATE** 1.20+
- ✅ **XFCE** 4.14+ (with plugin)
- ✅ **Budgie** 10.4+

### Setup Requirements

#### GNOME
GNOME removed native tray icon support. Install the AppIndicator extension:

```bash
# Ubuntu/Debian
sudo apt install gnome-shell-extension-appindicator

# Fedora
sudo dnf install gnome-shell-extension-appindicator

# Arch
sudo pacman -S gnome-shell-extension-appindicator-git
```

Then enable it:
```bash
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com
```

Or use GNOME Extensions app or website: https://extensions.gnome.org/extension/615/appindicator-support/

#### XFCE
Install the StatusNotifier Plugin:

```bash
# Ubuntu/Debian
sudo apt install xfce4-statusnotifier-plugin

# Arch
sudo pacman -S xfce4-statusnotifier-plugin
```

Add it to your panel: Right-click panel → Panel → Add New Items → Status Notifier Plugin

## Usage Examples

### Basic Setup

```rust
use drivesync::gui::tray::{TrayIcon, SyncStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create tray icon
    let tray = TrayIcon::new()?;

    // Set account info
    tray.set_account(Some("user@example.com".to_string()));

    // Set initial status
    tray.set_status(SyncStatus::Idle);

    // Run your app...
    Ok(())
}
```

### Setting Up Callbacks

```rust
use drivesync::gui::tray::TrayIcon;

let tray = TrayIcon::new()?;

// Pause/Resume callback
tray.on_pause_resume(|| {
    println!("User toggled sync!");
    // Toggle your sync engine here
});

// Open folder callback
tray.on_open_folder(|| {
    // Open sync folder with default file manager
    std::process::Command::new("xdg-open")
        .arg("/home/user/GoogleDrive")
        .spawn()
        .ok();
});

// Settings callback
tray.on_settings(|| {
    // Show settings window
});

// Quit callback
tray.on_quit(|| {
    std::process::exit(0);
});
```

### Updating Status

```rust
use drivesync::gui::tray::{StorageInfo, SyncStatus};

// Update sync status
tray.set_status(SyncStatus::Syncing);

// Update storage information
tray.set_storage_info(Some(StorageInfo {
    used_gb: 8.5,
    total_gb: 15.0,
    percent: 56.7,
}));

// Change account
tray.set_account(Some("another@email.com".to_string()));
```

### Desktop Notifications

```rust
use drivesync::gui::tray::NotificationManager;

let notifications = NotificationManager::new();

// Show sync complete
notifications.notify_sync_complete(15)?;

// Show error
notifications.notify_sync_error("Failed to upload file.txt")?;

// Show conflict
notifications.notify_conflict("document.docx")?;

// Show quota warning
notifications.notify_quota_warning(95.0)?;

// Custom notification
notifications.notify(
    "Custom Title",
    "Custom message",
    Some("custom-icon-name")
)?;
```

## Running the Example

Try the system tray example to see all features:

```bash
cargo run --example system_tray
```

This example demonstrates:
- Creating a tray icon
- Setting status and account info
- Handling menu callbacks
- Showing various notifications
- Simulating different sync states

## Troubleshooting

### Tray Icon Not Appearing

**GNOME**: Make sure AppIndicator extension is installed and enabled
```bash
gnome-extensions list | grep appindicator
```

**XFCE**: Ensure StatusNotifier Plugin is added to your panel

**Check if your DE supports system trays**:
```bash
# Check if StatusNotifierWatcher is available
qdbus org.kde.StatusNotifierWatcher /StatusNotifierWatcher
```

### Notifications Not Showing

Check if notification daemon is running:
```bash
# Most DEs use one of these
ps aux | grep -E 'notification|dunst|mako'
```

Install notification daemon if needed:
```bash
# Ubuntu/Debian
sudo apt install notification-daemon

# Arch
sudo pacman -S notification-daemon
# or dunst for minimal systems
sudo pacman -S dunst
```

### Icon Looks Wrong

DriveSync uses system theme icons. Make sure you have an icon theme installed:

```bash
# Ubuntu/Debian
sudo apt install adwaita-icon-theme

# Fedora
sudo dnf install adwaita-icon-theme

# Arch
sudo pacman -S adwaita-icon-theme
```

### Wayland Issues

Some Wayland compositors have limitations:
- **Sway**: Works with waybar or other StatusNotifier-compatible bars
- **Hyprland**: Use waybar with `tray` module
- **GNOME Wayland**: Use AppIndicator extension (same as X11)

## Best Practices

### Performance
- Update tray status only when it actually changes
- Batch multiple updates if possible
- Don't update more than once per second

### User Experience
- Use appropriate urgency levels for notifications
- Don't spam notifications - batch similar events
- Provide clear, actionable messages
- Use recognizable, system-standard icons

### Accessibility
- Ensure tooltip text is descriptive
- Don't rely solely on colors to convey status
- Provide menu alternatives for all tray functions

## API Reference

### TrayIcon

```rust
pub struct TrayIcon { /* ... */ }

impl TrayIcon {
    pub fn new() -> Result<Self>;
    pub fn set_status(&self, status: SyncStatus);
    pub fn set_account(&self, email: Option<String>);
    pub fn set_storage_info(&self, info: Option<StorageInfo>);
    pub fn on_pause_resume<F>(&self, callback: F);
    pub fn on_open_folder<F>(&self, callback: F);
    pub fn on_settings<F>(&self, callback: F);
    pub fn on_quit<F>(&self, callback: F);
}
```

### SyncStatus

```rust
pub enum SyncStatus {
    Idle,      // All files synced
    Syncing,   // Currently syncing
    Paused,    // Sync paused by user
    Error,     // Error occurred
    Offline,   // No internet connection
}
```

### NotificationManager

```rust
pub struct NotificationManager { /* ... */ }

impl NotificationManager {
    pub fn new() -> Self;
    pub fn notify_sync_complete(&self, files_synced: usize) -> Result<()>;
    pub fn notify_sync_error(&self, error: &str) -> Result<()>;
    pub fn notify_conflict(&self, filename: &str) -> Result<()>;
    pub fn notify_quota_warning(&self, percent_used: f64) -> Result<()>;
    pub fn notify_upload_complete(&self, filename: &str) -> Result<()>;
    pub fn notify_download_complete(&self, filename: &str) -> Result<()>;
    pub fn notify(&self, summary: &str, body: &str, icon: Option<&str>) -> Result<()>;
}
```

## Additional Resources

- [StatusNotifierItem Specification](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/)
- [ksni Rust Crate Documentation](https://docs.rs/ksni/)
- [notify-rust Documentation](https://docs.rs/notify-rust/)
- [freedesktop.org Notification Specification](https://specifications.freedesktop.org/notification-spec/)

## Support

If you encounter issues with system tray integration:
1. Check this guide's troubleshooting section
2. Verify your desktop environment is supported
3. Check [GitHub Issues](https://github.com/jiadanni/ldrive/issues)
4. Create a new issue with your DE and distribution info

---

**Note**: System tray and notification support varies by desktop environment. If you use a minimal window manager, you may need to install additional components.
