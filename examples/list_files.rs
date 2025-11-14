/// Example: List Google Drive Files
///
/// This example demonstrates how to list files from Google Drive.
///
/// Prerequisites: You must have already authenticated using the oauth_flow example.
///
/// Usage:
/// ```bash
/// cargo run --example list_files -- <YOUR_EMAIL> <CLIENT_ID> <CLIENT_SECRET>
/// ```

use drivesync::api::{AuthManager, DriveClient};
use std::env;

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
    if args.len() != 4 {
        eprintln!("Usage: {} <EMAIL> <CLIENT_ID> <CLIENT_SECRET>", args[0]);
        std::process::exit(1);
    }

    let email = args[1].clone();
    let client_id = args[2].clone();
    let client_secret = args[3].clone();

    // Create auth manager and retrieve credentials
    let auth = AuthManager::new(client_id, client_secret);

    println!("Retrieving credentials for {}...", email);
    let credentials = auth.get_valid_credentials(&email).await?;

    // Create Drive client
    let client = DriveClient::new(credentials);

    println!("\n=== Listing Your Google Drive Files ===\n");

    // List files
    match client.list_files(None, Some(20)).await {
        Ok(file_list) => {
            if file_list.files.is_empty() {
                println!("No files found in your Drive.");
            } else {
                println!("Found {} files:\n", file_list.files.len());

                for (index, file) in file_list.files.iter().enumerate() {
                    let file_type = if file.is_folder() {
                        "📁 Folder"
                    } else {
                        "📄 File"
                    };

                    let size_str = if let Some(size) = file.size {
                        format_size(size)
                    } else {
                        "N/A".to_string()
                    };

                    println!("{:2}. {} {}", index + 1, file_type, file.name);
                    println!("    ID: {}", file.id);
                    println!("    Size: {}", size_str);
                    println!("    Modified: {}", file.modified_time.format("%Y-%m-%d %H:%M:%S"));
                    println!();
                }

                if let Some(next_token) = file_list.next_page_token {
                    println!("Note: More files available. Use the next_page_token to fetch more.");
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing files: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
