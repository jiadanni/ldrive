/// Configuration management module
///
/// Handles application configuration including:
/// - User preferences
/// - Sync settings
/// - Network settings
/// - Multiple account management

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub accounts: Vec<AccountConfig>,
    pub sync: SyncConfig,
    pub network: NetworkConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub email: String,
    pub enabled: bool,
    pub sync_path: PathBuf,
    pub selective_folders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub conflict_resolution: ConflictResolution,
    pub auto_sync: bool,
    pub sync_hidden_files: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictResolution {
    Rename,
    Overwrite,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bandwidth_limit: u64, // 0 = unlimited, otherwise in bytes per second
    pub concurrent_uploads: usize,
    pub concurrent_downloads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub start_minimized: bool,
    pub show_notifications: bool,
    pub dark_mode: DarkMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DarkMode {
    Auto,
    Light,
    Dark,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            sync: SyncConfig::default(),
            network: NetworkConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            conflict_resolution: ConflictResolution::Rename,
            auto_sync: true,
            sync_hidden_files: false,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bandwidth_limit: 0,
            concurrent_uploads: 3,
            concurrent_downloads: 3,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            start_minimized: false,
            show_notifications: true,
            dark_mode: DarkMode::Auto,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "github", "drivesync")
            .ok_or_else(|| anyhow::anyhow!("Failed to determine config directory"))?;

        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "github", "drivesync")
            .ok_or_else(|| anyhow::anyhow!("Failed to determine data directory"))?;

        Ok(proj_dirs.data_dir().to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.accounts.len(), 0);
        assert!(config.sync.auto_sync);
    }
}
