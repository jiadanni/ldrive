/// Main application window with sync progress
///
/// Features:
/// - File list with sync status
/// - Progress bar showing current operation
/// - Control buttons (pause/resume, sync now)
/// - Statistics and account information

use gtk::prelude::*;
use gtk::{
    glib, Application, ApplicationWindow, Box, Button, HeaderBar, Label, ListBox, Orientation,
    ProgressBar, ScrolledWindow, Separator, Stack, StackSwitcher,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::sync::{EngineState, SyncProgress};

/// Main application window
pub struct MainWindow {
    window: ApplicationWindow,
    progress_bar: ProgressBar,
    status_label: Label,
    files_label: Label,
    operation_label: Label,
    pause_resume_btn: Button,
    sync_now_btn: Button,
    file_list: ListBox,
    // State
    engine_state: Arc<RwLock<EngineState>>,
}

impl MainWindow {
    /// Create a new main window
    pub fn new(app: &Application) -> Self {
        // Create main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("DriveSync for Linux")
            .default_width(900)
            .default_height(700)
            .build();

        // Create header bar
        let header_bar = HeaderBar::new();
        header_bar.set_show_title_buttons(true);

        // Add header bar buttons
        let menu_button = Button::from_icon_name("open-menu-symbolic");
        header_bar.pack_end(&menu_button);

        window.set_titlebar(Some(&header_bar));

        // Create main content with stack for multiple views
        let stack = Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);

        // Create stack switcher for navigation
        let stack_switcher = StackSwitcher::new();
        stack_switcher.set_stack(Some(&stack));
        header_bar.set_title_widget(Some(&stack_switcher));

        // Create sync view
        let sync_view = Self::create_sync_view();
        stack.add_titled(&sync_view.container, Some("sync"), "Sync");

        // Create files view
        let files_view = Self::create_files_view();
        stack.add_titled(&files_view.container, Some("files"), "Files");

        // Create settings view placeholder
        let settings_view = Self::create_settings_view();
        stack.add_titled(&settings_view, Some("settings"), "Settings");

        window.set_child(Some(&stack));

        Self {
            window,
            progress_bar: sync_view.progress_bar,
            status_label: sync_view.status_label,
            files_label: sync_view.files_label,
            operation_label: sync_view.operation_label,
            pause_resume_btn: sync_view.pause_resume_btn,
            sync_now_btn: sync_view.sync_now_btn,
            file_list: files_view.file_list,
            engine_state: Arc::new(RwLock::new(EngineState::Stopped)),
        }
    }

    /// Create sync status view
    fn create_sync_view() -> SyncViewComponents {
        let container = Box::new(Orientation::Vertical, 0);

        // Status section
        let status_box = Box::new(Orientation::Vertical, 12);
        status_box.set_margin_top(24);
        status_box.set_margin_bottom(24);
        status_box.set_margin_start(24);
        status_box.set_margin_end(24);

        // Account info
        let account_label = Label::new(Some("Account: user@example.com"));
        account_label.set_halign(gtk::Align::Start);
        account_label.add_css_class("title-3");
        status_box.append(&account_label);

        // Status indicator
        let status_label = Label::new(Some("Status: Idle"));
        status_label.set_halign(gtk::Align::Start);
        status_label.add_css_class("dim-label");
        status_box.append(&status_label);

        // Progress bar
        let progress_bar = ProgressBar::new();
        progress_bar.set_margin_top(12);
        progress_bar.set_show_text(true);
        progress_bar.set_text(Some("No active sync"));
        status_box.append(&progress_bar);

        // Operation label
        let operation_label = Label::new(Some(""));
        operation_label.set_halign(gtk::Align::Start);
        operation_label.add_css_class("dim-label");
        operation_label.set_margin_top(6);
        status_box.append(&operation_label);

        // Files label
        let files_label = Label::new(Some("Files synced: 0"));
        files_label.set_halign(gtk::Align::Start);
        files_label.add_css_class("caption");
        files_label.set_margin_top(6);
        status_box.append(&files_label);

        container.append(&status_box);

        // Separator
        let separator = Separator::new(Orientation::Horizontal);
        container.append(&separator);

        // Control buttons
        let control_box = Box::new(Orientation::Horizontal, 12);
        control_box.set_margin_top(24);
        control_box.set_margin_bottom(24);
        control_box.set_margin_start(24);
        control_box.set_margin_end(24);
        control_box.set_halign(gtk::Align::Center);

        let pause_resume_btn = Button::with_label("Pause Sync");
        pause_resume_btn.add_css_class("pill");
        control_box.append(&pause_resume_btn);

        let sync_now_btn = Button::with_label("Sync Now");
        sync_now_btn.add_css_class("suggested-action");
        sync_now_btn.add_css_class("pill");
        control_box.append(&sync_now_btn);

        container.append(&control_box);

        // Statistics section
        let stats_box = Box::new(Orientation::Vertical, 12);
        stats_box.set_margin_top(24);
        stats_box.set_margin_bottom(24);
        stats_box.set_margin_start(24);
        stats_box.set_margin_end(24);

        let stats_title = Label::new(Some("Statistics"));
        stats_title.set_halign(gtk::Align::Start);
        stats_title.add_css_class("title-4");
        stats_box.append(&stats_title);

        // Stats grid
        let stats_grid = gtk::Grid::new();
        stats_grid.set_row_spacing(8);
        stats_grid.set_column_spacing(12);
        stats_grid.set_margin_top(12);

        let labels = [
            ("Total Files:", "0"),
            ("Local Changes:", "0"),
            ("Remote Changes:", "0"),
            ("Conflicts:", "0"),
        ];

        for (row, (label_text, value_text)) in labels.iter().enumerate() {
            let label = Label::new(Some(*label_text));
            label.set_halign(gtk::Align::Start);
            label.add_css_class("dim-label");
            stats_grid.attach(&label, 0, row as i32, 1, 1);

            let value = Label::new(Some(*value_text));
            value.set_halign(gtk::Align::Start);
            value.add_css_class("title-4");
            stats_grid.attach(&value, 1, row as i32, 1, 1);
        }

        stats_box.append(&stats_grid);
        container.append(&stats_box);

        SyncViewComponents {
            container,
            progress_bar,
            status_label,
            files_label,
            operation_label,
            pause_resume_btn,
            sync_now_btn,
        }
    }

    /// Create files list view
    fn create_files_view() -> FilesViewComponents {
        let container = Box::new(Orientation::Vertical, 0);

        // Toolbar
        let toolbar = Box::new(Orientation::Horizontal, 12);
        toolbar.set_margin_top(12);
        toolbar.set_margin_bottom(12);
        toolbar.set_margin_start(12);
        toolbar.set_margin_end(12);

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search files..."));
        search_entry.set_hexpand(true);
        toolbar.append(&search_entry);

        let filter_btn = Button::from_icon_name("funnel-symbolic");
        toolbar.append(&filter_btn);

        container.append(&toolbar);

        // File list
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let file_list = ListBox::new();
        file_list.add_css_class("boxed-list");
        file_list.set_margin_top(12);
        file_list.set_margin_bottom(12);
        file_list.set_margin_start(12);
        file_list.set_margin_end(12);

        // Add placeholder
        let placeholder = Label::new(Some("No files to display"));
        placeholder.add_css_class("dim-label");
        placeholder.set_margin_top(48);
        placeholder.set_margin_bottom(48);
        file_list.set_placeholder(Some(&placeholder));

        scrolled.set_child(Some(&file_list));
        container.append(&scrolled);

        FilesViewComponents {
            container,
            file_list,
        }
    }

    /// Create settings view
    fn create_settings_view() -> Box {
        let container = Box::new(Orientation::Vertical, 24);
        container.set_margin_top(24);
        container.set_margin_bottom(24);
        container.set_margin_start(24);
        container.set_margin_end(24);

        let title = Label::new(Some("Settings"));
        title.add_css_class("title-1");
        title.set_halign(gtk::Align::Start);
        container.append(&title);

        // Sync settings
        let sync_group = Self::create_preference_group("Sync Settings");
        container.append(&sync_group);

        // Notification settings
        let notif_group = Self::create_preference_group("Notifications");
        container.append(&notif_group);

        // Account settings
        let account_group = Self::create_preference_group("Account");
        container.append(&account_group);

        container
    }

    /// Create a preference group
    fn create_preference_group(title: &str) -> Box {
        let group = Box::new(Orientation::Vertical, 12);

        let title_label = Label::new(Some(title));
        title_label.set_halign(gtk::Align::Start);
        title_label.add_css_class("title-4");
        group.append(&title_label);

        let list = ListBox::new();
        list.add_css_class("boxed-list");
        group.append(&list);

        group
    }

    /// Update progress display
    pub fn update_progress(&self, progress: &SyncProgress) {
        let fraction = if progress.total_files > 0 {
            progress.completed_files as f64 / progress.total_files as f64
        } else {
            0.0
        };

        self.progress_bar.set_fraction(fraction);

        if progress.total_files > 0 {
            let text = format!(
                "{}/{} files",
                progress.completed_files, progress.total_files
            );
            self.progress_bar.set_text(Some(&text));
        } else {
            self.progress_bar.set_text(Some("No active sync"));
        }

        self.operation_label
            .set_text(&progress.current_operation);

        if let Some(ref file) = progress.current_file {
            self.operation_label.set_text(&format!(
                "{}: {}",
                progress.current_operation, file
            ));
        }

        let files_text = format!(
            "Files synced: {} / {}",
            progress.completed_files, progress.total_files
        );
        self.files_label.set_text(&files_text);
    }

    /// Update engine state display
    pub fn update_state(&self, state: EngineState) {
        let status_text = match state {
            EngineState::Stopped => "Status: Stopped",
            EngineState::Running => "Status: Syncing",
            EngineState::Paused => "Status: Paused",
        };
        self.status_label.set_text(status_text);

        // Update pause/resume button
        let button_text = match state {
            EngineState::Paused => "Resume Sync",
            _ => "Pause Sync",
        };
        self.pause_resume_btn.set_label(button_text);

        // Update engine state
        let state_arc = self.engine_state.clone();
        glib::spawn_future_local(async move {
            *state_arc.write().await = state;
        });
    }

    /// Connect pause/resume button
    pub fn connect_pause_resume<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.pause_resume_btn.connect_clicked(move |_| {
            callback();
        });
    }

    /// Connect sync now button
    pub fn connect_sync_now<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.sync_now_btn.connect_clicked(move |_| {
            callback();
        });
    }

    /// Present the window
    pub fn present(&self) {
        self.window.present();
    }

    /// Get window reference
    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }
}

struct SyncViewComponents {
    container: Box,
    progress_bar: ProgressBar,
    status_label: Label,
    files_label: Label,
    operation_label: Label,
    pause_resume_btn: Button,
    sync_now_btn: Button,
}

struct FilesViewComponents {
    container: Box,
    file_list: ListBox,
}
