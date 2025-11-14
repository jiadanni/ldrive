/// System tray icon implementation using ksni
///
/// Provides a status icon in the system tray with:
/// - Different icons for different sync states
/// - Context menu with actions
/// - Tooltip with current status

use super::{Result, TrayError};
use ksni::{menu::*, Icon, Tray, TrayService};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Paused,
    Error,
    Offline,
}

impl SyncStatus {
    pub fn to_string(&self) -> &'static str {
        match self {
            SyncStatus::Idle => "Synced",
            SyncStatus::Syncing => "Syncing...",
            SyncStatus::Paused => "Paused",
            SyncStatus::Error => "Error",
            SyncStatus::Offline => "Offline",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            SyncStatus::Idle => "emblem-synchronizing-symbolic",
            SyncStatus::Syncing => "emblem-synchronizing",
            SyncStatus::Paused => "media-playback-pause-symbolic",
            SyncStatus::Error => "dialog-error-symbolic",
            SyncStatus::Offline => "network-offline-symbolic",
        }
    }
}

pub struct DriveSyncTray {
    status: Arc<Mutex<SyncStatus>>,
    account_email: Arc<Mutex<Option<String>>>,
    storage_info: Arc<Mutex<Option<StorageInfo>>>,
    on_pause_resume: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
    on_open_folder: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
    on_settings: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
    on_quit: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
}

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub used_gb: f64,
    pub total_gb: f64,
    pub percent: f64,
}

impl DriveSyncTray {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(SyncStatus::Idle)),
            account_email: Arc::new(Mutex::new(None)),
            storage_info: Arc::new(Mutex::new(None)),
            on_pause_resume: Arc::new(Mutex::new(None)),
            on_open_folder: Arc::new(Mutex::new(None)),
            on_settings: Arc::new(Mutex::new(None)),
            on_quit: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_status(&self, status: SyncStatus) {
        if let Ok(mut s) = self.status.lock() {
            *s = status;
        }
    }

    pub fn set_account(&self, email: Option<String>) {
        if let Ok(mut a) = self.account_email.lock() {
            *a = email;
        }
    }

    pub fn set_storage_info(&self, info: Option<StorageInfo>) {
        if let Ok(mut s) = self.storage_info.lock() {
            *s = info;
        }
    }

    pub fn on_pause_resume<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        if let Ok(mut cb) = self.on_pause_resume.lock() {
            *cb = Some(Box::new(callback));
        }
    }

    pub fn on_open_folder<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        if let Ok(mut cb) = self.on_open_folder.lock() {
            *cb = Some(Box::new(callback));
        }
    }

    pub fn on_settings<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        if let Ok(mut cb) = self.on_settings.lock() {
            *cb = Some(Box::new(callback));
        }
    }

    pub fn on_quit<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        if let Ok(mut cb) = self.on_quit.lock() {
            *cb = Some(Box::new(callback));
        }
    }

    fn get_tooltip(&self) -> String {
        let status = self.status.lock().ok().map(|s| *s).unwrap_or(SyncStatus::Offline);
        let account = self.account_email.lock().ok().and_then(|a| a.clone());
        let storage = self.storage_info.lock().ok().and_then(|s| s.clone());

        let mut tooltip = format!("DriveSync - {}", status.to_string());

        if let Some(email) = account {
            tooltip.push_str(&format!("\n{}", email));
        }

        if let Some(info) = storage {
            tooltip.push_str(&format!(
                "\nStorage: {:.1} GB / {:.1} GB ({:.0}%)",
                info.used_gb, info.total_gb, info.percent
            ));
        }

        tooltip
    }
}

impl Tray for DriveSyncTray {
    fn id(&self) -> String {
        "com.github.drivesync".to_string()
    }

    fn title(&self) -> String {
        "DriveSync".to_string()
    }

    fn icon_name(&self) -> String {
        let status = self.status.lock().ok().map(|s| *s).unwrap_or(SyncStatus::Idle);
        status.icon_name().to_string()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        // SVG icon data could be embedded here
        // For now, we rely on icon_name() and system theme icons
        vec![]
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            icon_name: self.icon_name(),
            title: self.title(),
            description: self.get_tooltip(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let status = self.status.lock().ok().map(|s| *s).unwrap_or(SyncStatus::Idle);
        let is_paused = status == SyncStatus::Paused;

        vec![
            MenuItem::Standard(StandardItem {
                label: if is_paused {
                    "Resume Sync".to_string()
                } else {
                    "Pause Sync".to_string()
                },
                icon_name: if is_paused {
                    "media-playback-start-symbolic".to_string()
                } else {
                    "media-playback-pause-symbolic".to_string()
                },
                activate: Box::new(|tray: &mut Self| {
                    if let Ok(cb) = tray.on_pause_resume.lock() {
                        if let Some(ref callback) = *cb {
                            callback();
                        }
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Open Sync Folder".to_string(),
                icon_name: "folder-open-symbolic".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    if let Ok(cb) = tray.on_open_folder.lock() {
                        if let Some(ref callback) = *cb {
                            callback();
                        }
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Settings".to_string(),
                icon_name: "preferences-system-symbolic".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    if let Ok(cb) = tray.on_settings.lock() {
                        if let Some(ref callback) = *cb {
                            callback();
                        }
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Quit".to_string(),
                icon_name: "application-exit-symbolic".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    if let Ok(cb) = tray.on_quit.lock() {
                        if let Some(ref callback) = *cb {
                            callback();
                        }
                    }
                }),
                ..Default::default()
            }),
        ]
    }
}

pub struct TrayIcon {
    service: TrayService<DriveSyncTray>,
    tray: Arc<DriveSyncTray>,
}

impl TrayIcon {
    pub fn new() -> Result<Self> {
        let tray = Arc::new(DriveSyncTray::new());
        let service = TrayService::new(tray.clone());

        Ok(Self { service, tray })
    }

    pub fn set_status(&self, status: SyncStatus) {
        self.tray.set_status(status);
        self.service.update();
    }

    pub fn set_account(&self, email: Option<String>) {
        self.tray.set_account(email);
        self.service.update();
    }

    pub fn set_storage_info(&self, info: Option<StorageInfo>) {
        self.tray.set_storage_info(info);
        self.service.update();
    }

    pub fn on_pause_resume<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.tray.on_pause_resume(callback);
    }

    pub fn on_open_folder<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.tray.on_open_folder(callback);
    }

    pub fn on_settings<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.tray.on_settings(callback);
    }

    pub fn on_quit<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.tray.on_quit(callback);
    }

    pub fn run(self) -> Result<()> {
        Ok(())
    }
}
