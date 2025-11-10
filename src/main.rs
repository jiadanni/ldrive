use gtk::prelude::*;
use gtk::{glib, Application};

mod api;
mod config;
mod gui;
mod storage;
mod sync;

const APP_ID: &str = "com.github.drivesync";

fn main() -> glib::ExitCode {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting DriveSync for Linux v{}", env!("CARGO_PKG_VERSION"));

    // Create GTK application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to application activate signal
    app.connect_activate(gui::build_ui);

    // Run the application
    app.run()
}
