/// Application coordinator
///
/// Integrates all components:
/// - Sync engine
/// - System tray
/// - Notifications
/// - Event handling

use crate::api::{AuthManager, DriveClient};
use crate::config::Config;
use crate::gui::tray::{NotificationManager, SyncStatus as TraySyncStatus, TrayIcon};
use crate::storage::Database;
use crate::sync::{EngineState, SyncEngine, SyncEvent, SyncProgress};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main application that coordinates all components
pub struct Application {
    sync_engine: Arc<SyncEngine>,
    tray_icon: Arc<TrayIcon>,
    notification_manager: Arc<NotificationManager>,
    config: Arc<Mutex<Config>>,
}

impl Application {
    /// Create a new application instance
    pub fn new(
        client: DriveClient,
        auth_manager: AuthManager,
        database: Database,
        config: Config,
        sync_root: PathBuf,
        account_email: String,
    ) -> Result<Self> {
        // Create sync engine
        let sync_engine = Arc::new(SyncEngine::new(
            client,
            auth_manager,
            database,
            config.clone(),
            sync_root.clone(),
            account_email.clone(),
        )?);

        // Create tray icon
        let tray_icon = Arc::new(TrayIcon::new()?);
        tray_icon.set_account(Some(account_email.clone()));
        tray_icon.set_status(TraySyncStatus::Idle);

        // Create notification manager
        let notification_manager = Arc::new(NotificationManager::new("DriveSync".to_string()));

        // Wire up callbacks
        let app = Self {
            sync_engine,
            tray_icon,
            notification_manager,
            config: Arc::new(Mutex::new(config)),
        };

        app.setup_callbacks();

        Ok(app)
    }

    /// Set up all event callbacks
    fn setup_callbacks(&self) {
        // Engine state changes -> Tray icon status
        let tray = self.tray_icon.clone();
        self.sync_engine.on_state_change(move |state| {
            let status = match state {
                EngineState::Stopped => TraySyncStatus::Offline,
                EngineState::Running => TraySyncStatus::Syncing,
                EngineState::Paused => TraySyncStatus::Paused,
            };
            tray.set_status(status);
        });

        // Sync events -> Notifications
        let notifications = self.notification_manager.clone();
        let tray = self.tray_icon.clone();
        self.sync_engine.on_sync_event(move |event| {
            match event {
                SyncEvent::SyncStarted => {
                    tray.set_status(TraySyncStatus::Syncing);
                }
                SyncEvent::SyncCompleted { files_synced } => {
                    tray.set_status(TraySyncStatus::Idle);
                    if files_synced > 0 {
                        let _ = notifications.notify_sync_complete(files_synced);
                    }
                }
                SyncEvent::FileUploaded { name } => {
                    let _ = notifications.notify_file_uploaded(&name);
                }
                SyncEvent::FileDownloaded { name } => {
                    let _ = notifications.notify_file_downloaded(&name);
                }
                SyncEvent::ConflictDetected { name, .. } => {
                    tray.set_status(TraySyncStatus::Error);
                    let _ = notifications.notify_conflict(&name);
                }
                SyncEvent::Error { message } => {
                    tray.set_status(TraySyncStatus::Error);
                    let _ = notifications.notify_error(&message);
                }
            }
        });

        // Tray pause/resume -> Engine control
        let engine = self.sync_engine.clone();
        self.tray_icon.on_pause_resume(move || {
            let engine = engine.clone();
            tokio::spawn(async move {
                let state = engine.get_state().await;
                match state {
                    EngineState::Running => {
                        let _ = engine.pause().await;
                    }
                    EngineState::Paused => {
                        let _ = engine.resume().await;
                    }
                    _ => {}
                }
            });
        });

        // Tray open folder -> Open sync directory
        let sync_root = self.sync_engine.clone();
        self.tray_icon.on_open_folder(move || {
            // TODO: Open sync folder in file manager
            tracing::info!("Open folder requested");
        });

        // Tray settings -> Open settings dialog
        self.tray_icon.on_settings(|| {
            // TODO: Open settings dialog
            tracing::info!("Settings requested");
        });

        // Tray quit -> Stop engine and exit
        let engine = self.sync_engine.clone();
        self.tray_icon.on_quit(move || {
            let engine = engine.clone();
            tokio::spawn(async move {
                let _ = engine.stop().await;
                std::process::exit(0);
            });
        });
    }

    /// Start the application
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting DriveSync application");

        // Start sync engine
        self.sync_engine.start().await?;

        tracing::info!("DriveSync started successfully");
        Ok(())
    }

    /// Stop the application
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping DriveSync application");

        // Stop sync engine
        self.sync_engine.stop().await?;

        tracing::info!("DriveSync stopped");
        Ok(())
    }

    /// Pause synchronization
    pub async fn pause(&self) -> Result<()> {
        self.sync_engine.pause().await?;
        Ok(())
    }

    /// Resume synchronization
    pub async fn resume(&self) -> Result<()> {
        self.sync_engine.resume().await?;
        Ok(())
    }

    /// Trigger manual sync
    pub async fn sync_now(&self) -> Result<()> {
        self.sync_engine.sync_now().await?;
        Ok(())
    }

    /// Get current sync progress
    pub async fn get_progress(&self) -> SyncProgress {
        self.sync_engine.get_progress().await
    }

    /// Get reference to sync engine
    pub fn sync_engine(&self) -> &Arc<SyncEngine> {
        &self.sync_engine
    }

    /// Get reference to tray icon
    pub fn tray_icon(&self) -> &Arc<TrayIcon> {
        &self.tray_icon
    }

    /// Run the application (blocking)
    pub fn run(self) -> Result<()> {
        // Keep the application running
        // In a real application, this would be integrated with GTK main loop
        self.tray_icon.run()?;
        Ok(())
    }
}
