/// File system monitoring using inotify

use crate::sync::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc;

pub struct FileMonitor {
    watcher: Option<RecommendedWatcher>,
    event_receiver: mpsc::UnboundedReceiver<Event>,
}

impl FileMonitor {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            watcher: None,
            event_receiver: rx,
        }
    }

    /// Start monitoring a directory
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        // TODO: Implement file monitoring
        todo!("Implement file monitoring")
    }

    /// Stop monitoring
    pub fn stop(&mut self) -> Result<()> {
        // TODO: Implement monitoring stop
        todo!("Implement monitoring stop")
    }

    /// Get next file system event
    pub async fn next_event(&mut self) -> Option<Event> {
        self.event_receiver.recv().await
    }
}
