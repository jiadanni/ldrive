/// Conflict resolution strategies
///
/// Detects and resolves conflicts when files are modified both
/// locally and remotely between sync operations.

use crate::config::ConflictResolution;
use crate::storage::FileState;
use crate::sync::Result;
use chrono::Utc;

pub struct ConflictResolver {
    strategy: ConflictResolution,
}

impl ConflictResolver {
    pub fn new(strategy: ConflictResolution) -> Self {
        Self { strategy }
    }

    /// Detect if there's a conflict between local and remote files
    pub fn detect_conflict(&self, local: &FileState, remote: &FileState) -> bool {
        // A conflict exists if:
        // 1. Both files have been modified since last sync
        // 2. The checksums don't match (if available)
        // 3. The modification times are different

        // If we have checksums and they match, no conflict
        if let (Some(local_md5), Some(remote_md5)) = (&local.md5_checksum, &remote.md5_checksum) {
            if local_md5 == remote_md5 {
                return false;
            }
        }

        // Check if both have been modified since last sync
        if let Some(last_sync) = local.last_sync {
            let local_modified_since_sync = local.modified_time > last_sync;
            let remote_modified_since_sync = remote.modified_time > last_sync;

            if local_modified_since_sync && remote_modified_since_sync {
                return true;
            }
        }

        // If no last sync time, compare modification times
        // This is a conflict if times don't match
        local.modified_time != remote.modified_time
    }

    /// Resolve a conflict between local and remote files
    pub async fn resolve(
        &self,
        local: &FileState,
        remote: &FileState,
    ) -> Result<ResolutionAction> {
        tracing::info!(
            "Resolving conflict for file: {} (strategy: {:?})",
            local.name,
            self.strategy
        );

        match self.strategy {
            ConflictResolution::Rename => {
                // Keep both versions by renaming the local file
                let new_name = self.generate_conflict_name(&local.name);
                Ok(ResolutionAction::Rename(new_name))
            }
            ConflictResolution::Overwrite => {
                // Always keep the remote version
                Ok(ResolutionAction::KeepRemote)
            }
            ConflictResolution::Ask => {
                // Defer to user
                Ok(ResolutionAction::Ask)
            }
        }
    }

    /// Choose which version to keep based on modification time
    pub fn choose_newer(
        &self,
        local: &FileState,
        remote: &FileState,
    ) -> ResolutionAction {
        if local.modified_time > remote.modified_time {
            tracing::debug!("Local file is newer, keeping local");
            ResolutionAction::KeepLocal
        } else {
            tracing::debug!("Remote file is newer, keeping remote");
            ResolutionAction::KeepRemote
        }
    }

    /// Generate a conflict name for a file
    fn generate_conflict_name(&self, original_name: &str) -> String {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        if let Some((name, ext)) = original_name.rsplit_once('.') {
            format!("{} (conflicted copy {} - {}).{}", name, timestamp, hostname, ext)
        } else {
            format!("{} (conflicted copy {} - {})", original_name, timestamp, hostname)
        }
    }

    /// Set resolution strategy
    pub fn set_strategy(&mut self, strategy: ConflictResolution) {
        self.strategy = strategy;
    }

    /// Get current strategy
    pub fn strategy(&self) -> ConflictResolution {
        self.strategy
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionAction {
    /// Keep the local version
    KeepLocal,
    /// Keep the remote version
    KeepRemote,
    /// Rename local file to avoid conflict
    Rename(String),
    /// Ask the user what to do
    Ask,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_file(name: &str, modified: chrono::DateTime<Utc>) -> FileState {
        FileState {
            id: "test123".to_string(),
            name: name.to_string(),
            path: format!("/test/{}", name),
            mime_type: "text/plain".to_string(),
            size: Some(100),
            md5_checksum: None,
            modified_time: modified,
            parent_id: None,
            is_folder: false,
            sync_status: crate::storage::SyncStatus::Synced,
            last_sync: Some(Utc::now() - Duration::hours(1)),
        }
    }

    #[test]
    fn test_detect_conflict_with_matching_checksums() {
        let resolver = ConflictResolver::new(ConflictResolution::Rename);
        let now = Utc::now();

        let mut local = create_test_file("test.txt", now);
        local.md5_checksum = Some("abc123".to_string());

        let mut remote = create_test_file("test.txt", now + Duration::seconds(10));
        remote.md5_checksum = Some("abc123".to_string());

        assert!(!resolver.detect_conflict(&local, &remote));
    }

    #[test]
    fn test_detect_conflict_with_different_checksums() {
        let resolver = ConflictResolver::new(ConflictResolution::Rename);
        let base_time = Utc::now() - Duration::hours(2);

        let mut local = create_test_file("test.txt", base_time + Duration::minutes(30));
        local.md5_checksum = Some("abc123".to_string());
        local.last_sync = Some(base_time);

        let mut remote = create_test_file("test.txt", base_time + Duration::minutes(45));
        remote.md5_checksum = Some("def456".to_string());
        remote.last_sync = Some(base_time);

        assert!(resolver.detect_conflict(&local, &remote));
    }

    #[tokio::test]
    async fn test_resolve_with_rename_strategy() {
        let resolver = ConflictResolver::new(ConflictResolution::Rename);
        let now = Utc::now();

        let local = create_test_file("document.txt", now);
        let remote = create_test_file("document.txt", now + Duration::seconds(10));

        let action = resolver.resolve(&local, &remote).await.unwrap();

        match action {
            ResolutionAction::Rename(name) => {
                assert!(name.contains("document"));
                assert!(name.contains("conflicted copy"));
                assert!(name.ends_with(".txt"));
            }
            _ => panic!("Expected Rename action"),
        }
    }

    #[tokio::test]
    async fn test_resolve_with_overwrite_strategy() {
        let resolver = ConflictResolver::new(ConflictResolution::Overwrite);
        let now = Utc::now();

        let local = create_test_file("document.txt", now);
        let remote = create_test_file("document.txt", now + Duration::seconds(10));

        let action = resolver.resolve(&local, &remote).await.unwrap();

        assert_eq!(action, ResolutionAction::KeepRemote);
    }

    #[test]
    fn test_choose_newer() {
        let resolver = ConflictResolver::new(ConflictResolution::Rename);
        let now = Utc::now();

        let local = create_test_file("test.txt", now);
        let remote = create_test_file("test.txt", now - Duration::seconds(10));

        let action = resolver.choose_newer(&local, &remote);
        assert_eq!(action, ResolutionAction::KeepLocal);

        let action = resolver.choose_newer(&remote, &local);
        assert_eq!(action, ResolutionAction::KeepRemote);
    }

    #[test]
    fn test_generate_conflict_name() {
        let resolver = ConflictResolver::new(ConflictResolution::Rename);

        let name = resolver.generate_conflict_name("document.txt");
        assert!(name.contains("document"));
        assert!(name.contains("conflicted copy"));
        assert!(name.ends_with(".txt"));

        let name_no_ext = resolver.generate_conflict_name("README");
        assert!(name_no_ext.contains("README"));
        assert!(name_no_ext.contains("conflicted copy"));
    }
}
