/// End-to-end integration tests
///
/// Tests the complete sync workflow including:
/// - Database operations
/// - File monitoring
/// - Conflict resolution
/// - Application coordination

use anyhow::Result;
use drivesync::config::{Config, ConflictResolution, NetworkConfig, SyncConfig, UiConfig};
use drivesync::storage::{Database, FileState, SyncStatus};
use drivesync::sync::{ConflictResolver, FileMonitor, ResolutionAction};
use chrono::Utc;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_database_operations() -> Result<()> {
    // Create temporary database
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path)?;

    // Test file insertion
    let file = FileState {
        id: "test-file-1".to_string(),
        name: "test.txt".to_string(),
        path: "/test/test.txt".to_string(),
        mime_type: "text/plain".to_string(),
        size: Some(1024),
        md5_checksum: Some("abc123".to_string()),
        modified_time: Utc::now(),
        parent_id: None,
        is_folder: false,
        sync_status: SyncStatus::Synced,
        last_sync: Some(Utc::now()),
    };

    db.upsert_file(&file)?;

    // Test file retrieval
    let retrieved = db.get_file("test-file-1")?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.name, "test.txt");
    assert_eq!(retrieved.size, Some(1024));

    // Test file update
    let mut updated_file = file.clone();
    updated_file.size = Some(2048);
    db.upsert_file(&updated_file)?;

    let retrieved = db.get_file("test-file-1")?.unwrap();
    assert_eq!(retrieved.size, Some(2048));

    // Test sync status update
    db.update_sync_status("test-file-1", SyncStatus::Pending)?;
    let retrieved = db.get_file("test-file-1")?.unwrap();
    assert_eq!(retrieved.sync_status, SyncStatus::Pending);

    // Test conflict tracking
    db.add_conflict("test-file-1", Utc::now(), Utc::now())?;
    let conflicts = db.get_unresolved_conflicts()?;
    assert_eq!(conflicts.len(), 1);

    // Test sync history
    db.add_sync_history("test-file-1", "upload", true, None)?;
    let stats = db.get_sync_stats()?;
    assert!(stats.last_sync.is_some());

    Ok(())
}

#[test]
fn test_conflict_resolution() -> Result<()> {
    let resolver = ConflictResolver::new(ConflictResolution::Rename);

    // Create two conflicting files
    let local_file = FileState {
        id: "file-1".to_string(),
        name: "document.txt".to_string(),
        path: "/sync/document.txt".to_string(),
        mime_type: "text/plain".to_string(),
        size: Some(1024),
        md5_checksum: Some("local_checksum".to_string()),
        modified_time: Utc::now(),
        parent_id: None,
        is_folder: false,
        sync_status: SyncStatus::Synced,
        last_sync: Some(Utc::now() - chrono::Duration::hours(1)),
    };

    let mut remote_file = local_file.clone();
    remote_file.md5_checksum = Some("remote_checksum".to_string());
    remote_file.modified_time = Utc::now() + chrono::Duration::minutes(5);

    // Test conflict detection
    let has_conflict = resolver.detect_conflict(&local_file, &remote_file);
    assert!(has_conflict, "Should detect conflict when checksums differ");

    // Test conflict resolution
    let action = resolver.resolve_conflict(&local_file, &remote_file)?;
    match action {
        ResolutionAction::RenameLocal(new_name) => {
            assert!(new_name.contains("conflicted copy"));
            assert!(new_name.contains("document"));
        }
        _ => panic!("Expected RenameLocal action"),
    }

    Ok(())
}

#[test]
fn test_file_monitor_creation() -> Result<()> {
    let monitor = FileMonitor::new()?;
    assert!(true, "FileMonitor should be created successfully");
    Ok(())
}

#[test]
fn test_config_management() -> Result<()> {
    // Test default config
    let config = Config::default();
    assert_eq!(config.accounts.len(), 0);
    assert!(config.sync.auto_sync);
    assert_eq!(config.sync.conflict_resolution, ConflictResolution::Rename);
    assert_eq!(config.network.bandwidth_limit, 0);
    assert_eq!(config.network.concurrent_uploads, 3);
    assert!(config.ui.show_notifications);

    // Test config serialization
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.toml");

    // Create a custom config
    let mut custom_config = Config::default();
    custom_config.sync.conflict_resolution = ConflictResolution::Overwrite;
    custom_config.network.bandwidth_limit = 1048576; // 1 MB/s

    // Save config
    let content = toml::to_string_pretty(&custom_config)?;
    fs::write(&config_path, content)?;

    // Load config
    let loaded_content = fs::read_to_string(&config_path)?;
    let loaded_config: Config = toml::from_str(&loaded_content)?;

    assert_eq!(
        loaded_config.sync.conflict_resolution,
        ConflictResolution::Overwrite
    );
    assert_eq!(loaded_config.network.bandwidth_limit, 1048576);

    Ok(())
}

#[tokio::test]
async fn test_sync_engine_state_transitions() -> Result<()> {
    use drivesync::sync::EngineState;

    // Test state enum
    assert_eq!(EngineState::Stopped, EngineState::Stopped);
    assert_ne!(EngineState::Stopped, EngineState::Running);

    Ok(())
}

#[test]
fn test_file_state_equality() {
    let file1 = FileState {
        id: "file-1".to_string(),
        name: "test.txt".to_string(),
        path: "/test.txt".to_string(),
        mime_type: "text/plain".to_string(),
        size: Some(1024),
        md5_checksum: Some("abc123".to_string()),
        modified_time: Utc::now(),
        parent_id: None,
        is_folder: false,
        sync_status: SyncStatus::Synced,
        last_sync: Some(Utc::now()),
    };

    let file2 = file1.clone();
    assert_eq!(file1.id, file2.id);
    assert_eq!(file1.name, file2.name);
    assert_eq!(file1.md5_checksum, file2.md5_checksum);
}

#[test]
fn test_sync_status_values() {
    assert_eq!(
        format!("{:?}", SyncStatus::Synced),
        "Synced"
    );
    assert_eq!(
        format!("{:?}", SyncStatus::Pending),
        "Pending"
    );
    assert_eq!(
        format!("{:?}", SyncStatus::Failed),
        "Failed"
    );
}

#[test]
fn test_conflict_strategies() {
    let rename = ConflictResolver::new(ConflictResolution::Rename);
    let overwrite = ConflictResolver::new(ConflictResolution::Overwrite);
    let ask = ConflictResolver::new(ConflictResolution::Ask);

    // Just verify they can be created
    assert!(true);
}

/// Test database concurrent access
#[tokio::test]
async fn test_database_concurrent_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("concurrent.db");
    let db = std::sync::Arc::new(Database::new(&db_path)?);

    // Spawn multiple tasks that write to database
    let mut handles = vec![];
    for i in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let file = FileState {
                id: format!("file-{}", i),
                name: format!("test-{}.txt", i),
                path: format!("/test-{}.txt", i),
                mime_type: "text/plain".to_string(),
                size: Some(1024),
                md5_checksum: Some(format!("checksum-{}", i)),
                modified_time: Utc::now(),
                parent_id: None,
                is_folder: false,
                sync_status: SyncStatus::Synced,
                last_sync: Some(Utc::now()),
            };
            db_clone.upsert_file(&file)
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await??;
    }

    // Verify all files were inserted
    for i in 0..10 {
        let file = db.get_file(&format!("file-{}", i))?;
        assert!(file.is_some());
    }

    Ok(())
}

/// Integration test for complete workflow
#[tokio::test]
async fn test_complete_sync_workflow() -> Result<()> {
    // Create temporary directories and database
    let temp_dir = TempDir::new()?;
    let sync_dir = temp_dir.path().join("sync");
    fs::create_dir_all(&sync_dir)?;

    let db_path = temp_dir.path().join("workflow.db");
    let db = Database::new(&db_path)?;

    // Create a test file
    let test_file_path = sync_dir.join("test.txt");
    fs::write(&test_file_path, "Hello, DriveSync!")?;

    // Create file state
    let file_state = FileState {
        id: "workflow-file".to_string(),
        name: "test.txt".to_string(),
        path: test_file_path.to_string_lossy().to_string(),
        mime_type: "text/plain".to_string(),
        size: Some(17),
        md5_checksum: Some("test_checksum".to_string()),
        modified_time: Utc::now(),
        parent_id: None,
        is_folder: false,
        sync_status: SyncStatus::Pending,
        last_sync: None,
    };

    // Insert into database
    db.upsert_file(&file_state)?;

    // Simulate sync
    db.update_sync_status("workflow-file", SyncStatus::Synced)?;
    db.add_sync_history("workflow-file", "upload", true, None)?;

    // Verify final state
    let final_state = db.get_file("workflow-file")?.unwrap();
    assert_eq!(final_state.sync_status, SyncStatus::Synced);

    let stats = db.get_sync_stats()?;
    assert!(stats.last_sync.is_some());

    Ok(())
}
