/// Core synchronization engine
///
/// Coordinates all sync operations between local filesystem and Google Drive
///
/// Architecture:
/// - Monitors local filesystem for changes
/// - Polls Google Drive for remote changes
/// - Detects and resolves conflicts
/// - Executes sync operations (upload/download)
/// - Maintains sync state in database

use crate::api::{AuthManager, DriveClient};
use crate::config::{Config, ConflictResolution};
use crate::storage::{Database, FileState, SyncStatus};
use crate::sync::{ConflictResolver, FileEvent, FileEventKind, FileMonitor, ResolutionAction, Result, SyncError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{self, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Stopped,
    Running,
    Paused,
}

pub struct SyncEngine {
    client: Arc<Mutex<DriveClient>>,
    auth_manager: Arc<AuthManager>,
    database: Arc<Mutex<Database>>,
    config: Arc<RwLock<Config>>,
    monitor: Arc<Mutex<FileMonitor>>,
    resolver: Arc<ConflictResolver>,
    state: Arc<Mutex<EngineState>>,
    should_stop: Arc<AtomicBool>,
    sync_root: PathBuf,
    account_email: String,
}

impl SyncEngine {
    pub fn new(
        client: DriveClient,
        auth_manager: AuthManager,
        database: Database,
        config: Config,
        sync_root: PathBuf,
        account_email: String,
    ) -> Result<Self> {
        let monitor = FileMonitor::new()?;
        let conflict_strategy = config.sync.conflict_resolution;
        let resolver = ConflictResolver::new(conflict_strategy);

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            auth_manager: Arc::new(auth_manager),
            database: Arc::new(Mutex::new(database)),
            config: Arc::new(RwLock::new(config)),
            monitor: Arc::new(Mutex::new(monitor)),
            resolver: Arc::new(resolver),
            state: Arc::new(Mutex::new(EngineState::Stopped)),
            should_stop: Arc::new(AtomicBool::new(false)),
            sync_root,
            account_email,
        })
    }

    /// Start the sync engine
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        if *state != EngineState::Stopped {
            return Err(SyncError::SyncFailed("Sync engine already running".to_string()));
        }

        tracing::info!("Starting sync engine for: {}", self.account_email);

        // Start file monitor
        let mut monitor = self.monitor.lock().await;
        monitor.watch(&self.sync_root)?;
        drop(monitor);

        *state = EngineState::Running;
        self.should_stop.store(false, Ordering::Relaxed);
        drop(state);

        // Spawn background sync loop
        self.spawn_sync_loop();

        // Spawn file monitor processor
        self.spawn_monitor_processor();

        tracing::info!("Sync engine started successfully");
        Ok(())
    }

    /// Stop the sync engine
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping sync engine");

        self.should_stop.store(true, Ordering::Relaxed);

        let mut state = self.state.lock().await;
        *state = EngineState::Stopped;
        drop(state);

        // Stop file monitor
        let mut monitor = self.monitor.lock().await;
        monitor.stop()?;

        tracing::info!("Sync engine stopped");
        Ok(())
    }

    /// Pause synchronization
    pub async fn pause(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        if *state == EngineState::Running {
            *state = EngineState::Paused;
            tracing::info!("Sync engine paused");
        }
        Ok(())
    }

    /// Resume synchronization
    pub async fn resume(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        if *state == EngineState::Paused {
            *state = EngineState::Running;
            tracing::info!("Sync engine resumed");
        }
        Ok(())
    }

    /// Check if engine is running
    pub async fn is_running(&self) -> bool {
        let state = self.state.lock().await;
        *state == EngineState::Running
    }

    /// Get current engine state
    pub async fn get_state(&self) -> EngineState {
        *self.state.lock().await
    }

    /// Trigger manual sync
    pub async fn sync_now(&self) -> Result<()> {
        tracing::info!("Manual sync triggered");
        self.perform_sync().await
    }

    /// Main sync loop - runs periodically
    fn spawn_sync_loop(&self) {
        let engine = self.clone_arc();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                if engine.should_stop.load(Ordering::Relaxed) {
                    break;
                }

                let state = *engine.state.lock().await;
                if state != EngineState::Running {
                    continue;
                }

                if let Err(e) = engine.perform_sync().await {
                    tracing::error!("Sync error: {}", e);
                }
            }
        });
    }

    /// File monitor event processor
    fn spawn_monitor_processor(&self) {
        let engine = self.clone_arc();

        tokio::spawn(async move {
            loop {
                if engine.should_stop.load(Ordering::Relaxed) {
                    break;
                }

                let state = *engine.state.lock().await;
                if state != EngineState::Running {
                    time::sleep(Duration::from_millis(500)).await;
                    continue;
                }

                let mut monitor = engine.monitor.lock().await;
                let event = monitor.next_event().await;
                drop(monitor);

                if let Some(event) = event {
                    if let Err(e) = engine.handle_file_event(event).await {
                        tracing::error!("Error handling file event: {}", e);
                    }
                }
            }
        });
    }

    /// Handle a file system event
    async fn handle_file_event(&self, event: FileEvent) -> Result<()> {
        tracing::debug!("File event: {:?}", event);

        match event.kind {
            FileEventKind::Created => self.handle_local_create(&event.path).await,
            FileEventKind::Modified => self.handle_local_modify(&event.path).await,
            FileEventKind::Deleted => self.handle_local_delete(&event.path).await,
            FileEventKind::Renamed { from, to } => self.handle_local_rename(&from, &to).await,
        }
    }

    /// Handle local file creation
    async fn handle_local_create(&self, path: &Path) -> Result<()> {
        tracing::info!("Local file created: {:?}", path);

        // Upload to Drive
        let client = self.client.lock().await;
        let file = client.upload_file(path, None, None).await?;
        drop(client);

        // Update database
        let db = self.database.lock().await;
        db.upsert_file(&FileState {
            id: file.id,
            name: file.name,
            path: path.to_string_lossy().to_string(),
            mime_type: file.mime_type,
            size: file.size,
            md5_checksum: file.md5_checksum,
            modified_time: file.modified_time,
            parent_id: None,
            is_folder: file.is_folder(),
            sync_status: SyncStatus::Synced,
            last_sync: Some(chrono::Utc::now()),
        })?;

        db.add_sync_history(&file.id, "upload", true, None)?;

        Ok(())
    }

    /// Handle local file modification
    async fn handle_local_modify(&self, path: &Path) -> Result<()> {
        tracing::info!("Local file modified: {:?}", path);

        let path_str = path.to_string_lossy().to_string();

        // Get file from database
        let db = self.database.lock().await;
        let file_state = db.get_file_by_path(&path_str)?;
        drop(db);

        if let Some(state) = file_state {
            // Update file on Drive
            let client = self.client.lock().await;
            let _ = client.update_file(&state.id, path).await?;
            drop(client);

            // Update database
            let db = self.database.lock().await;
            db.update_sync_status(&state.id, SyncStatus::Synced)?;
            db.add_sync_history(&state.id, "update", true, None)?;
        }

        Ok(())
    }

    /// Handle local file deletion
    async fn handle_local_delete(&self, path: &Path) -> Result<()> {
        tracing::info!("Local file deleted: {:?}", path);

        let path_str = path.to_string_lossy().to_string();

        // Get file from database
        let db = self.database.lock().await;
        let file_state = db.get_file_by_path(&path_str)?;
        drop(db);

        if let Some(state) = file_state {
            // Delete from Drive
            let client = self.client.lock().await;
            let _ = client.delete_file(&state.id).await?;
            drop(client);

            // Remove from database
            let db = self.database.lock().await;
            db.delete_file(&state.id)?;
            db.add_sync_history(&state.id, "delete", true, None)?;
        }

        Ok(())
    }

    /// Handle local file rename
    async fn handle_local_rename(&self, _from: &Path, _to: &Path) -> Result<()> {
        // TODO: Implement rename handling
        tracing::warn!("Rename handling not yet implemented");
        Ok(())
    }

    /// Perform a full sync cycle
    async fn perform_sync(&self) -> Result<()> {
        tracing::debug!("Performing sync cycle");

        // Refresh credentials if needed
        self.ensure_valid_credentials().await?;

        // Fetch remote changes
        self.sync_remote_changes().await?;

        // Process pending local changes
        self.sync_pending_local_changes().await?;

        Ok(())
    }

    /// Ensure credentials are valid, refresh if needed
    async fn ensure_valid_credentials(&self) -> Result<()> {
        let credentials = self.auth_manager
            .get_valid_credentials(&self.account_email)
            .await?;

        let mut client = self.client.lock().await;
        client.update_credentials(credentials);

        Ok(())
    }

    /// Sync changes from Google Drive
    async fn sync_remote_changes(&self) -> Result<()> {
        // List files from Drive
        let client = self.client.lock().await;
        let file_list = client.list_files(None, Some(100)).await?;
        drop(client);

        let db = self.database.lock().await;

        for remote_file in file_list.files {
            // Check if file exists locally
            if let Some(local_file) = db.get_file(&remote_file.id)? {
                // Check for conflicts
                if self.resolver.detect_conflict(&local_file, &FileState {
                    id: remote_file.id.clone(),
                    name: remote_file.name.clone(),
                    path: local_file.path.clone(),
                    mime_type: remote_file.mime_type.clone(),
                    size: remote_file.size,
                    md5_checksum: remote_file.md5_checksum.clone(),
                    modified_time: remote_file.modified_time,
                    parent_id: remote_file.parents.first().cloned(),
                    is_folder: remote_file.is_folder(),
                    sync_status: SyncStatus::Synced,
                    last_sync: local_file.last_sync,
                }) {
                    // Handle conflict
                    tracing::warn!("Conflict detected for file: {}", remote_file.name);
                    db.add_conflict(&remote_file.id, local_file.modified_time, remote_file.modified_time)?;
                } else if remote_file.modified_time > local_file.modified_time {
                    // Download newer remote version
                    drop(db);
                    self.download_file(&remote_file.id, &local_file.path).await?;
                    let db = self.database.lock().await;
                    db.update_sync_status(&remote_file.id, SyncStatus::Synced)?;
                }
            } else {
                // New file, download it
                let download_path = self.sync_root.join(&remote_file.name);
                drop(db);
                self.download_file(&remote_file.id, &download_path.to_string_lossy()).await?;
                let db = self.database.lock().await;
            }
        }

        Ok(())
    }

    /// Download a file from Drive
    async fn download_file(&self, file_id: &str, local_path: &str) -> Result<()> {
        let client = self.client.lock().await;
        client.download_file(file_id, Path::new(local_path)).await?;
        Ok(())
    }

    /// Sync pending local changes
    async fn sync_pending_local_changes(&self) -> Result<()> {
        let db = self.database.lock().await;
        let pending_files = db.get_files_by_status(SyncStatus::Pending)?;
        drop(db);

        for file in pending_files {
            // Upload pending file
            match self.upload_file(&file).await {
                Ok(_) => {
                    let db = self.database.lock().await;
                    db.update_sync_status(&file.id, SyncStatus::Synced)?;
                }
                Err(e) => {
                    tracing::error!("Failed to upload {}: {}", file.name, e);
                    let db = self.database.lock().await;
                    db.update_sync_status(&file.id, SyncStatus::Error)?;
                }
            }
        }

        Ok(())
    }

    /// Upload a file to Drive
    async fn upload_file(&self, file_state: &FileState) -> Result<()> {
        let client = self.client.lock().await;
        client.upload_file(Path::new(&file_state.path), None, Some(&file_state.name)).await?;
        Ok(())
    }

    /// Helper to clone Arc references for spawning tasks
    fn clone_arc(&self) -> Arc<Self> {
        Arc::new(Self {
            client: self.client.clone(),
            auth_manager: self.auth_manager.clone(),
            database: self.database.clone(),
            config: self.config.clone(),
            monitor: self.monitor.clone(),
            resolver: self.resolver.clone(),
            state: self.state.clone(),
            should_stop: self.should_stop.clone(),
            sync_root: self.sync_root.clone(),
            account_email: self.account_email.clone(),
        })
    }
}
