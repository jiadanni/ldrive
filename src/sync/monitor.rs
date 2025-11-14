/// File system monitoring using inotify
///
/// Monitors a directory tree for file changes and reports them
/// for synchronization processing.

use crate::sync::{Result, SyncError};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

#[derive(Debug, Clone)]
pub struct FileEvent {
    pub kind: FileEventKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEventKind {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

pub struct FileMonitor {
    watcher: Option<RecommendedWatcher>,
    event_receiver: mpsc::UnboundedReceiver<FileEvent>,
    watched_paths: Vec<PathBuf>,
}

impl FileMonitor {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        Ok(Self {
            watcher: None,
            event_receiver: rx,
            watched_paths: Vec::new(),
        })
    }

    /// Start monitoring a directory
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        tracing::info!("Starting file monitor for: {:?}", path);

        if !path.exists() {
            return Err(SyncError::SyncFailed(format!(
                "Path does not exist: {:?}",
                path
            )));
        }

        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<notify::Result<Event>>();

        // Create watcher with event handler
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = event_tx.send(res);
            },
            notify::Config::default()
                .with_poll_interval(Duration::from_secs(1))
                .with_compare_contents(true),
        )
        .map_err(|e| SyncError::SyncFailed(format!("Failed to create watcher: {}", e)))?;

        self.watcher = Some(watcher);

        // Watch the path recursively
        if let Some(ref mut watcher) = self.watcher {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| SyncError::SyncFailed(format!("Failed to watch path: {}", e)))?;
        }

        self.watched_paths.push(path.to_path_buf());

        // Spawn task to process notify events
        let file_event_tx = self.event_receiver.sender().clone();
        let watch_path = path.to_path_buf();

        tokio::spawn(async move {
            let mut debounce_map: std::collections::HashMap<PathBuf, time::Instant> =
                std::collections::HashMap::new();
            let debounce_duration = Duration::from_millis(100);

            while let Some(res) = event_rx.recv().await {
                match res {
                    Ok(event) => {
                        if let Some(file_event) = Self::process_event(event, &watch_path) {
                            // Simple debouncing: ignore events that are too close together
                            let now = time::Instant::now();
                            let should_send = debounce_map
                                .get(&file_event.path)
                                .map(|last_time| now.duration_since(*last_time) > debounce_duration)
                                .unwrap_or(true);

                            if should_send {
                                debounce_map.insert(file_event.path.clone(), now);
                                if file_event_tx.send(file_event).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("File monitor error: {}", e);
                    }
                }
            }
        });

        tracing::info!("File monitor started successfully");
        Ok(())
    }

    /// Process a notify event into our simplified FileEvent
    fn process_event(event: Event, watch_path: &Path) -> Option<FileEvent> {
        // Filter out irrelevant events
        if event.paths.is_empty() {
            return None;
        }

        let path = &event.paths[0];

        // Ignore temporary files and hidden files (optional)
        if let Some(filename) = path.file_name() {
            let name = filename.to_string_lossy();
            if name.starts_with('.') || name.ends_with('~') || name.ends_with(".tmp") {
                return None;
            }
        }

        // Only process events within our watched path
        if !path.starts_with(watch_path) {
            return None;
        }

        match event.kind {
            EventKind::Create(_) => Some(FileEvent {
                kind: FileEventKind::Created,
                path: path.clone(),
            }),
            EventKind::Modify(_) => Some(FileEvent {
                kind: FileEventKind::Modified,
                path: path.clone(),
            }),
            EventKind::Remove(_) => Some(FileEvent {
                kind: FileEventKind::Deleted,
                path: path.clone(),
            }),
            EventKind::Any => {
                // Generic event, treat as modified
                Some(FileEvent {
                    kind: FileEventKind::Modified,
                    path: path.clone(),
                })
            }
            _ => None,
        }
    }

    /// Stop monitoring
    pub fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping file monitor");

        if let Some(mut watcher) = self.watcher.take() {
            for path in &self.watched_paths {
                let _ = watcher.unwatch(path);
            }
        }

        self.watched_paths.clear();
        tracing::info!("File monitor stopped");
        Ok(())
    }

    /// Get next file system event
    pub async fn next_event(&mut self) -> Option<FileEvent> {
        self.event_receiver.recv().await
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.watcher.is_some()
    }

    /// Get list of watched paths
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
}

impl Drop for FileMonitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_monitor_creation() {
        let monitor = FileMonitor::new();
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_file_monitor_watch() {
        let temp_dir = TempDir::new().unwrap();
        let mut monitor = FileMonitor::new().unwrap();

        let result = monitor.watch(temp_dir.path());
        assert!(result.is_ok());
        assert!(monitor.is_monitoring());
    }

    #[tokio::test]
    async fn test_file_monitor_detects_create() {
        let temp_dir = TempDir::new().unwrap();
        let mut monitor = FileMonitor::new().unwrap();
        monitor.watch(temp_dir.path()).unwrap();

        // Give the monitor time to initialize
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Create a file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Wait for event
        let event = tokio::time::timeout(Duration::from_secs(2), monitor.next_event())
            .await
            .ok()
            .flatten();

        assert!(event.is_some());
        if let Some(e) = event {
            assert_eq!(e.kind, FileEventKind::Created);
        }
    }
}
