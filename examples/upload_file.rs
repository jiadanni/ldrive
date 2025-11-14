/// Example: Upload File to Google Drive
///
/// This example demonstrates how to upload a file to Google Drive.
///
/// Prerequisites: You must have already authenticated using the oauth_flow example.
///
/// Usage:
/// ```bash
/// cargo run --example upload_file -- <YOUR_EMAIL> <CLIENT_ID> <CLIENT_SECRET> <FILE_PATH>
/// ```

use drivesync::api::{AuthManager, DriveClient};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Get arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <EMAIL> <CLIENT_ID> <CLIENT_SECRET> <FILE_PATH>", args[0]);
        std::process::exit(1);
    }

    let email = args[1].clone();
    let client_id = args[2].clone();
    let client_secret = args[3].clone();
    let file_path = Path::new(&args[4]);

    // Verify file exists
    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        std::process::exit(1);
    }

    // Create auth manager and retrieve credentials
    let auth = AuthManager::new(client_id, client_secret);

    println!("Retrieving credentials for {}...", email);
    let credentials = auth.get_valid_credentials(&email).await?;

    // Create Drive client
    let client = DriveClient::new(credentials);

    println!("\n=== Uploading File to Google Drive ===\n");
    println!("File: {}", file_path.display());

    // Upload file
    match client.upload_file(file_path, None, None).await {
        Ok(file) => {
            println!("\nUpload successful!");
            println!("  File Name: {}", file.name);
            println!("  File ID: {}", file.id);
            if let Some(link) = file.web_view_link {
                println!("  View Link: {}", link);
            }
            if let Some(size) = file.size {
                println!("  Size: {} bytes", size);
            }
        }
        Err(e) => {
            eprintln!("\nUpload failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
