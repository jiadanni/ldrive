/// Complete integration example
///
/// Demonstrates:
/// - Full application setup with all components
/// - Sync engine + tray icon + notifications
/// - Event-driven architecture
/// - Progress tracking and status updates

use anyhow::Result;
use drivesync::api::{AuthManager, Credentials, DriveClient};
use drivesync::app::Application;
use drivesync::config::Config;
use drivesync::storage::Database;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("=== DriveSync Integration Example ===\n");

    // Load configuration
    let config = Config::load().unwrap_or_default();
    let data_dir = Config::data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    // Create database
    let db_path = data_dir.join("drivesync.db");
    let database = Database::new(&db_path)?;
    println!("✓ Database initialized at {:?}", db_path);

    // Create auth manager
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .expect("GOOGLE_CLIENT_ID environment variable required");
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .expect("GOOGLE_CLIENT_SECRET environment variable required");

    let auth_manager = AuthManager::new(client_id, client_secret)?;
    println!("✓ Auth manager created");

    // Check for existing credentials or perform OAuth flow
    let account_email = "user@example.com".to_string(); // In production, get from auth flow
    let credentials = match auth_manager.get_credentials(&account_email) {
        Ok(creds) => {
            println!("✓ Using existing credentials");
            creds
        }
        Err(_) => {
            println!("No existing credentials found. Performing OAuth flow...");
            println!("\n1. Visit this URL to authorize the application:");

            let (auth_url, pkce_challenge) = auth_manager.get_auth_url()?;
            println!("{}\n", auth_url);

            println!("2. After authorization, paste the code here:");
            let mut code = String::new();
            std::io::stdin().read_line(&mut code)?;
            let code = code.trim();

            println!("Exchanging code for credentials...");
            let credentials = auth_manager.exchange_code(code, pkce_challenge).await?;

            // Store credentials
            auth_manager.store_credentials(&credentials.email, &credentials)?;
            println!("✓ Credentials stored");

            credentials
        }
    };

    // Create Drive client
    let client = DriveClient::new(credentials);
    println!("✓ Drive client created");

    // Set up sync directory
    let sync_root = dirs::home_dir()
        .expect("Could not determine home directory")
        .join("DriveSync");
    std::fs::create_dir_all(&sync_root)?;
    println!("✓ Sync directory: {:?}", sync_root);

    // Create application
    println!("\nInitializing application...");
    let app = Application::new(
        client,
        auth_manager,
        database,
        config,
        sync_root.clone(),
        account_email.clone(),
    )?;
    println!("✓ Application initialized");

    // Set up additional event handlers for demo
    let sync_engine = app.sync_engine().clone();
    sync_engine.on_progress(|progress| {
        if !progress.current_operation.is_empty() {
            println!(
                "[Progress] {} ({}/{})",
                progress.current_operation, progress.completed_files, progress.total_files
            );
            if let Some(ref file) = progress.current_file {
                println!("           Current file: {}", file);
            }
        }
    });

    // Start the application
    println!("\n=== Starting Sync Engine ===\n");
    app.start().await?;

    println!("✓ Sync engine started and monitoring: {:?}", sync_root);
    println!("\nThe application is now running!");
    println!("- System tray icon should be visible");
    println!("- File changes in {:?} will be synced", sync_root);
    println!("- Notifications will appear for sync events");
    println!("\nTray actions:");
    println!("  • Pause/Resume - Toggle synchronization");
    println!("  • Open Folder - Open sync directory");
    println!("  • Settings - Configure DriveSync (TODO)");
    println!("  • Quit - Stop sync and exit");

    println!("\n=== Testing Manual Sync ===\n");
    sleep(Duration::from_secs(2)).await;

    println!("Triggering manual sync...");
    app.sync_now().await?;
    println!("✓ Manual sync completed");

    println!("\n=== Monitoring for Changes ===\n");
    println!("Try the following:");
    println!("1. Create a file in {:?}", sync_root);
    println!("2. Modify an existing file");
    println!("3. Delete a file");
    println!("\nPress Ctrl+C to stop...\n");

    // Keep running until interrupted
    let app = Arc::new(app);
    let app_clone = app.clone();

    // Handle Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("\n\nShutting down...");
        if let Err(e) = app_clone.stop().await {
            eprintln!("Error stopping application: {}", e);
        }
        std::process::exit(0);
    });

    // Periodic status updates
    loop {
        sleep(Duration::from_secs(10)).await;

        let progress = app.get_progress().await;
        let state = app.sync_engine().get_state().await;

        println!(
            "[Status] Engine: {:?} | Operation: {} | Files: {}/{}",
            state,
            progress.current_operation,
            progress.completed_files,
            progress.total_files
        );
    }
}
