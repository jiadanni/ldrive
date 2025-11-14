/// Complete GUI application example
///
/// Demonstrates:
/// - GTK4 main window with sync progress
/// - System tray integration
/// - Real-time progress updates
/// - Event-driven UI updates

use anyhow::Result;
use drivesync::api::{AuthManager, Credentials, DriveClient};
use drivesync::app::Application as SyncApp;
use drivesync::config::Config;
use drivesync::gui::MainWindow;
use drivesync::storage::Database;
use gtk::glib;
use gtk::prelude::*;
use std::sync::Arc;

const APP_ID: &str = "com.github.drivesync.example";

fn main() -> glib::ExitCode {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Create GTK application
    let gtk_app = gtk::Application::builder().application_id(APP_ID).build();

    gtk_app.connect_activate(build_ui);

    // Run the application
    gtk_app.run()
}

fn build_ui(app: &gtk::Application) {
    // Create main window
    let main_window = Arc::new(MainWindow::new(app));
    main_window.present();

    // Initialize sync application asynchronously
    let main_window_clone = main_window.clone();
    glib::spawn_future_local(async move {
        match initialize_sync_app(main_window_clone).await {
            Ok(_) => {
                tracing::info!("Sync application initialized successfully");
            }
            Err(e) => {
                tracing::error!("Failed to initialize sync application: {}", e);
                let dialog = gtk::AlertDialog::builder()
                    .message("Failed to initialize")
                    .detail(format!("Error: {}", e))
                    .build();
                dialog.show(None::<&gtk::Window>);
            }
        }
    });
}

async fn initialize_sync_app(main_window: Arc<MainWindow>) -> Result<()> {
    // Load configuration
    let config = Config::load().unwrap_or_default();
    let data_dir = Config::data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    // Create database
    let db_path = data_dir.join("drivesync.db");
    let database = Database::new(&db_path)?;

    // Create auth manager
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .unwrap_or_else(|_| "YOUR_CLIENT_ID".to_string());
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .unwrap_or_else(|_| "YOUR_CLIENT_SECRET".to_string());

    let auth_manager = AuthManager::new(client_id, client_secret)?;

    // For demo purposes, use mock credentials
    // In production, this would be from OAuth flow
    let credentials = Credentials {
        access_token: "demo_token".to_string(),
        refresh_token: Some("demo_refresh".to_string()),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        email: "user@example.com".to_string(),
    };

    let client = DriveClient::new(credentials.clone());

    // Set up sync directory
    let sync_root = dirs::home_dir()
        .expect("Could not determine home directory")
        .join("DriveSync");
    std::fs::create_dir_all(&sync_root)?;

    // Create sync application
    let sync_app = Arc::new(SyncApp::new(
        client,
        auth_manager,
        database,
        config,
        sync_root,
        credentials.email,
    )?);

    // Connect sync engine callbacks to UI updates
    let sync_engine = sync_app.sync_engine().clone();
    let window_clone = main_window.clone();
    sync_engine.on_state_change(move |state| {
        let window = window_clone.clone();
        glib::spawn_future_local(async move {
            window.update_state(state);
        });
    });

    let window_clone = main_window.clone();
    sync_engine.on_progress(move |progress| {
        let window = window_clone.clone();
        glib::spawn_future_local(async move {
            window.update_progress(&progress);
        });
    });

    // Connect UI buttons to sync engine actions
    let engine_clone = sync_engine.clone();
    main_window.connect_pause_resume(move || {
        let engine = engine_clone.clone();
        glib::spawn_future_local(async move {
            let state = engine.get_state().await;
            match state {
                drivesync::sync::EngineState::Running => {
                    let _ = engine.pause().await;
                }
                drivesync::sync::EngineState::Paused => {
                    let _ = engine.resume().await;
                }
                _ => {}
            }
        });
    });

    let engine_clone = sync_engine.clone();
    main_window.connect_sync_now(move || {
        let engine = engine_clone.clone();
        glib::spawn_future_local(async move {
            match engine.sync_now().await {
                Ok(_) => tracing::info!("Manual sync completed"),
                Err(e) => tracing::error!("Manual sync failed: {}", e),
            }
        });
    });

    // Start the sync engine
    sync_app.start().await?;

    tracing::info!("Sync application started");

    Ok(())
}
