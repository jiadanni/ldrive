/// Example: System Tray Integration
///
/// This example demonstrates the system tray functionality including:
/// - Status icon in system tray
/// - Context menu with actions
/// - Desktop notifications
/// - Status updates
///
/// Usage:
/// ```bash
/// cargo run --example system_tray
/// ```

use drivesync::gui::tray::{NotificationManager, StorageInfo, SyncStatus, TrayIcon};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("=== DriveSync System Tray Example ===\n");
    println!("This example demonstrates the system tray integration.");
    println!("Look for the DriveSync icon in your system tray!\n");

    // Create notification manager
    let notifications = NotificationManager::new();

    // Create tray icon
    println!("Creating system tray icon...");
    let tray = Arc::new(
        TrayIcon::new()
            .map_err(|e| anyhow::anyhow!("Failed to create tray icon: {}", e))?,
    );

    // Set up callbacks
    let tray_clone = tray.clone();
    let notif_clone = notifications.clone();
    tray.on_pause_resume(move || {
        println!("Pause/Resume clicked");
        let _ = notif_clone.notify("Sync Status", "Toggled sync state", Some("emblem-synchronizing"));

        // In a real app, this would toggle the actual sync state
        // For demo purposes, we'll just toggle between Paused and Syncing
        tray_clone.set_status(SyncStatus::Syncing);
    });

    let notif_clone = notifications.clone();
    tray.on_open_folder(move || {
        println!("Open folder clicked");
        let _ = notif_clone.notify("Opening Folder", "Opening Google Drive folder...", Some("folder-open"));

        // In a real app, this would open the sync folder
        // For demo: xdg-open ~/GoogleDrive
    });

    let notif_clone = notifications.clone();
    tray.on_settings(move || {
        println!("Settings clicked");
        let _ = notif_clone.notify("Settings", "Opening settings...", Some("preferences-system"));

        // In a real app, this would open the settings dialog
    });

    tray.on_quit(|| {
        println!("Quit clicked");
        std::process::exit(0);
    });

    // Set initial account info
    tray.set_account(Some("user@example.com".to_string()));
    tray.set_storage_info(Some(StorageInfo {
        used_gb: 8.5,
        total_gb: 15.0,
        percent: 56.7,
    }));

    // Show welcome notification
    notifications.notify_auth_success("user@example.com")?;

    println!("\nSystem tray is now active!");
    println!("Try the following:");
    println!("  1. Click the tray icon to see the menu");
    println!("  2. Try Pause/Resume sync");
    println!("  3. Try opening the folder");
    println!("  4. Check the tooltip by hovering over the icon");
    println!("\nDemonstrating status changes...\n");

    // Simulate various states
    let states = vec![
        (SyncStatus::Idle, "Idle - All files synced", 3),
        (SyncStatus::Syncing, "Syncing files...", 5),
        (SyncStatus::Idle, "Sync complete!", 3),
        (SyncStatus::Paused, "Sync paused", 3),
        (SyncStatus::Syncing, "Resuming sync...", 3),
        (SyncStatus::Error, "Sync error occurred", 3),
        (SyncStatus::Idle, "Error recovered", 3),
        (SyncStatus::Offline, "Connection lost", 3),
        (SyncStatus::Syncing, "Back online, syncing...", 3),
        (SyncStatus::Idle, "All synced", 0),
    ];

    for (status, message, sleep_secs) in states {
        println!("Status: {:?} - {}", status, message);
        tray.set_status(status);

        // Show appropriate notification
        match status {
            SyncStatus::Syncing => {
                let _ = notifications.notify("Syncing", message, Some("emblem-synchronizing"));
            }
            SyncStatus::Error => {
                let _ = notifications.notify_sync_error(message);
            }
            SyncStatus::Offline => {
                let _ = notifications.notify_offline();
            }
            SyncStatus::Idle if message.contains("complete") => {
                let _ = notifications.notify_sync_complete(12);
            }
            SyncStatus::Idle if message.contains("online") => {
                let _ = notifications.notify_online();
            }
            _ => {}
        }

        if sleep_secs > 0 {
            time::sleep(Duration::from_secs(sleep_secs)).await;
        }
    }

    // Simulate file operations
    println!("\nSimulating file upload...");
    tray.set_status(SyncStatus::Syncing);
    notifications.notify_upload_complete("document.pdf")?;
    time::sleep(Duration::from_secs(2)).await;

    println!("Simulating file download...");
    notifications.notify_download_complete("image.png")?;
    time::sleep(Duration::from_secs(2)).await;

    println!("Simulating conflict...");
    notifications.notify_conflict("report.docx")?;
    time::sleep(Duration::from_secs(3)).await;

    // Update storage info
    println!("\nUpdating storage information...");
    tray.set_storage_info(Some(StorageInfo {
        used_gb: 13.8,
        total_gb: 15.0,
        percent: 92.0,
    }));
    notifications.notify_quota_warning(92.0)?;

    println!("\nTray icon will remain active.");
    println!("Press Ctrl+C to exit or use the Quit menu item.\n");

    // Keep the application running
    loop {
        time::sleep(Duration::from_secs(1)).await;
    }
}
