/// GTK4 user interface module
///
/// Provides:
/// - Main application window
/// - System tray integration
/// - Settings dialog
/// - Account management UI
/// - File browser

pub mod tray;
pub mod window;

pub use tray::{NotificationManager, SyncStatus, TrayIcon};
pub use window::MainWindow;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, HeaderBar, Label, Orientation};

pub fn build_ui(app: &Application) {
    // Create main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("DriveSync for Linux")
        .default_width(800)
        .default_height(600)
        .build();

    // Create header bar
    let header_bar = HeaderBar::new();
    header_bar.set_show_title_buttons(true);
    window.set_titlebar(Some(&header_bar));

    // Create main content
    let content_box = Box::new(Orientation::Vertical, 12);
    content_box.set_margin_top(12);
    content_box.set_margin_bottom(12);
    content_box.set_margin_start(12);
    content_box.set_margin_end(12);

    // Welcome message
    let welcome_label = Label::new(Some("Welcome to DriveSync for Linux"));
    welcome_label.add_css_class("title-1");
    content_box.append(&welcome_label);

    let subtitle = Label::new(Some("Free, open source Google Drive client"));
    subtitle.add_css_class("dim-label");
    content_box.append(&subtitle);

    // Add account button
    let add_account_btn = Button::with_label("Add Google Account");
    add_account_btn.add_css_class("suggested-action");
    add_account_btn.add_css_class("pill");
    add_account_btn.set_halign(gtk::Align::Center);
    add_account_btn.set_margin_top(24);

    add_account_btn.connect_clicked(|_| {
        println!("Add account clicked - OAuth flow will be implemented");
    });

    content_box.append(&add_account_btn);

    // Status label
    let status_label = Label::new(Some("No accounts configured"));
    status_label.add_css_class("dim-label");
    status_label.set_margin_top(12);
    content_box.append(&status_label);

    window.set_child(Some(&content_box));
    window.present();
}
