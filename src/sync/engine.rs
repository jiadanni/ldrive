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

/// Callback for engine state changes
pub type StateChangeCallback = Arc<dyn Fn(EngineState) + Send + Sync>;

/// Callback for sync progress updates
pub type ProgressCallback = Arc<dyn Fn(SyncProgress) + Send + Sync>;

/// Callback for sync events
pub type SyncEventCallback = Arc<dyn Fn(SyncEvent) + Send + Sync>;

/// Progress information for ongoing sync operations
#[derive(Debug, Clone)]
pub struct SyncProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub current_operation: String,
    pub current_file: Option<String>,
}

/// Events that occur during sync operations
#[derive(Debug, Clone)]
pub enum SyncEvent {
    SyncStarted,
    SyncCompleted { files_synced: usize },
    FileUploaded { name: String },
    FileDownloaded { name: String },
    ConflictDetected { file_id: String, name: String },
    Error { message: String },
}

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
    // Callbacks
    on_state_change: Arc<Mutex<Option<StateChangeCallback>>>,
    on_progress: Arc<Mutex<Option<ProgressCallback>>>,
    on_sync_event: Arc<Mutex<Option<SyncEventCallback>>>,
    // Progress tracking
    progress: Arc<Mutex<SyncProgress>>,
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
            on_state_change: Arc::new(Mutex::new(None)),
            on_progress: Arc::new(Mutex::new(None)),
            on_sync_event: Arc::new(Mutex::new(None)),
            progress: Arc::new(Mutex::new(SyncProgress {
                total_files: 0,
                completed_files: 0,
                current_operation: String::new(),
                current_file: None,
            })),
        })
    }

    /// Set callback for state changes
    pub fn on_state_change<F>(&self, callback: F)
    where
        F: Fn(EngineState) + Send + Sync + 'static,
    {
        if let Ok(mut cb) = self.on_state_change.lock() {
            *cb = Some(Arc::new(callback));
        }
    }

    /// Set callback for progress updates
    pub fn on_progress<F>(&self, callback: F)
    where
        F: Fn(SyncProgress) + Send + Sync + 'static,
    {
        if let Ok(mut cb) = self.on_progress.lock() {
            *cb = Some(Arc::new(callback));
        }
    }

    /// Set callback for sync events
    pub fn on_sync_event<F>(&self, callback: F)
    where
        F: Fn(SyncEvent) + Send + Sync + 'static,
    {
        if let Ok(mut cb) = self.on_sync_event.lock() {
            *cb = Some(Arc::new(callback));
        }
    }

    /// Get current progress
    pub async fn get_progress(&self) -> SyncProgress {
        self.progress.lock().await.clone()
    }

    /// Emit state change event
    fn emit_state_change(&self, state: EngineState) {
        if let Ok(cb) = self.on_state_change.lock() {
            if let Some(ref callback) = *cb {
                callback(state);
            }
        }
    }

    /// Emit progress update
    fn emit_progress(&self, progress: SyncProgress) {
        if let Ok(cb) = self.on_progress.lock() {
            if let Some(ref callback) = *cb {
                callback(progress);
            }
        }
    }

    /// Emit sync event
    fn emit_sync_event(&self, event: SyncEvent) {
        if let Ok(cb) = self.on_sync_event.lock() {
            if let Some(ref callback) = *cb {
                callback(event);
            }
        }
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
        self.emit_state_change(EngineState::Running);
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
        self.emit_state_change(EngineState::Stopped);
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
            self.emit_state_change(EngineState::Paused);
            tracing::info!("Sync engine paused");
        }
        Ok(())
    }

    /// Resume synchronization
    pub async fn resume(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        if *state == EngineState::Paused {
            *state = EngineState::Running;
            self.emit_state_change(EngineState::Running);
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

    /// Update sync interval (requires restart to take effect)
    pub async fn set_sync_interval(&self, seconds: u64) {
        let mut config = self.config.write().await;
        config.sync.sync_interval_seconds = seconds;
        tracing::info!("Sync interval updated to {} seconds (restart required)", seconds);
    }

    /// Main sync loop - runs periodically
    fn spawn_sync_loop(&self) {
        let engine = self.clone_arc();

        tokio::spawn(async move {
            // Get sync interval from config
            let interval_secs = engine.config.read().await.sync.sync_interval_seconds;
            let mut interval = time::interval(Duration::from_secs(interval_secs));

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
        let file_name = file.name.clone();
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
        drop(db);

        // Emit upload event
        self.emit_sync_event(SyncEvent::FileUploaded { name: file_name });

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

        // Emit sync started event
        self.emit_sync_event(SyncEvent::SyncStarted);

        // Update progress
        let mut progress = self.progress.lock().await;
        progress.current_operation = "Checking credentials".to_string();
        progress.current_file = None;
        let progress_clone = progress.clone();
        drop(progress);
        self.emit_progress(progress_clone);

        // Refresh credentials if needed
        if let Err(e) = self.ensure_valid_credentials().await {
            self.emit_sync_event(SyncEvent::Error {
                message: format!("Credential error: {}", e),
            });
            return Err(e);
        }

        // Fetch remote changes
        let mut progress = self.progress.lock().await;
        progress.current_operation = "Syncing from Google Drive".to_string();
        let progress_clone = progress.clone();
        drop(progress);
        self.emit_progress(progress_clone);

        let remote_result = self.sync_remote_changes().await;
        if let Err(e) = &remote_result {
            self.emit_sync_event(SyncEvent::Error {
                message: format!("Remote sync error: {}", e),
            });
        }
        remote_result?;

        // Process pending local changes
        let mut progress = self.progress.lock().await;
        progress.current_operation = "Uploading local changes".to_string();
        let progress_clone = progress.clone();
        drop(progress);
        self.emit_progress(progress_clone);

        let local_result = self.sync_pending_local_changes().await;
        if let Err(e) = &local_result {
            self.emit_sync_event(SyncEvent::Error {
                message: format!("Local sync error: {}", e),
            });
        }
        local_result?;

        // Emit completion event
        let progress = self.progress.lock().await;
        let files_synced = progress.completed_files;
        drop(progress);

        self.emit_sync_event(SyncEvent::SyncCompleted { files_synced });

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

                    // Emit conflict event
                    self.emit_sync_event(SyncEvent::ConflictDetected {
                        file_id: remote_file.id.clone(),
                        name: remote_file.name.clone(),
                    });
                } else if remote_file.modified_time > local_file.modified_time {
                    // Download newer remote version
                    let file_name = remote_file.name.clone();
                    drop(db);
                    self.download_file(&remote_file.id, &local_file.path).await?;
                    let db = self.database.lock().await;
                    db.update_sync_status(&remote_file.id, SyncStatus::Synced)?;

                    // Emit download event
                    self.emit_sync_event(SyncEvent::FileDownloaded { name: file_name });
                }
            } else {
                // New file, download it
                let download_path = self.sync_root.join(&remote_file.name);
                let file_name = remote_file.name.clone();
                drop(db);
                self.download_file(&remote_file.id, &download_path.to_string_lossy()).await?;
                let db = self.database.lock().await;

                // Emit download event
                self.emit_sync_event(SyncEvent::FileDownloaded { name: file_name });
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
            on_state_change: self.on_state_change.clone(),
            on_progress: self.on_progress.clone(),
            on_sync_event: self.on_sync_event.clone(),
            progress: self.progress.clone(),
        })
    }
}
