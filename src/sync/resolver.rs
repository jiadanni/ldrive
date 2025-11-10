/// Conflict resolution strategies

use crate::config::ConflictResolution;
use crate::storage::FileState;
use crate::sync::Result;

pub struct ConflictResolver {
    strategy: ConflictResolution,
}

impl ConflictResolver {
    pub fn new(strategy: ConflictResolution) -> Self {
        Self { strategy }
    }

    /// Resolve a conflict between local and remote files
    pub async fn resolve(
        &self,
        local: &FileState,
        remote: &FileState,
    ) -> Result<ResolutionAction> {
        // TODO: Implement conflict resolution
        todo!("Implement conflict resolution")
    }

    /// Set resolution strategy
    pub fn set_strategy(&mut self, strategy: ConflictResolution) {
        self.strategy = strategy;
    }
}

#[derive(Debug, Clone)]
pub enum ResolutionAction {
    KeepLocal,
    KeepRemote,
    Rename(String),
    Ask,
}
