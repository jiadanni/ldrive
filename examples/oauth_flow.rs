/// Example: OAuth 2.0 Authentication Flow
///
/// This example demonstrates how to authenticate with Google Drive API using OAuth 2.0.
///
/// Usage:
/// ```bash
/// cargo run --example oauth_flow -- <CLIENT_ID> <CLIENT_SECRET>
/// ```
///
/// You can get your CLIENT_ID and CLIENT_SECRET from Google Cloud Console:
/// https://console.cloud.google.com/apis/credentials

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

    // Get client credentials from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <CLIENT_ID> <CLIENT_SECRET>", args[0]);
        eprintln!("\nGet your credentials from:");
        eprintln!("https://console.cloud.google.com/apis/credentials");
        std::process::exit(1);
    }

    let client_id = args[1].clone();
    let client_secret = args[2].clone();

    // Create auth manager
    let auth = AuthManager::new(client_id, client_secret);

    println!("=== DriveSync OAuth 2.0 Flow Example ===\n");

    // Step 1: Generate authorization URL
    println!("Step 1: Generating authorization URL...");
    let (auth_url, _csrf_token, pkce_challenge) = auth.get_auth_url()?;

    println!("\nPlease visit this URL to authorize the application:");
    println!("{}\n", auth_url);
    println!("After authorizing, you'll be redirected to http://localhost:8080");
    println!("The application will automatically capture the authorization code.\n");

    // Step 2: Start local server to listen for callback
    println!("Step 2: Waiting for authorization callback...");
    let code = auth.listen_for_callback().await?;
    println!("Received authorization code!\n");

    // Step 3: Exchange code for tokens
    println!("Step 3: Exchanging authorization code for access tokens...");
    let credentials = auth.exchange_code(&code, pkce_challenge).await?;
    println!("Successfully obtained access tokens!");
    println!("  Account email: {}", credentials.email);
    println!("  Token expires at: {}\n", credentials.expires_at);

    // Step 4: Store credentials securely
    println!("Step 4: Storing credentials in system keyring...");
    auth.store_credentials(&credentials.email, &credentials)?;
    println!("Credentials stored securely!\n");

    // Step 5: Test API access
    println!("Step 5: Testing Google Drive API access...");
    let client = DriveClient::new(credentials);

    // Get account information
    match client.get_about().await {
        Ok(about) => {
            println!("Successfully connected to Google Drive!");
            println!("\nAccount Information:");
            if let Some(user) = about.user {
                println!("  Name: {}", user.display_name);
                println!("  Email: {}", user.email_address);
            }
            println!("\nStorage Quota:");
            let quota = about.storage_quota;
            let used_gb = quota.usage as f64 / 1_000_000_000.0;
            let limit_gb = quota.limit as f64 / 1_000_000_000.0;
            let percent = (quota.usage as f64 / quota.limit as f64) * 100.0;
            println!("  Used: {:.2} GB / {:.2} GB ({:.1}%)", used_gb, limit_gb, percent);
        }
        Err(e) => {
            eprintln!("Failed to access Google Drive API: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n=== Authentication Complete ===");
    println!("\nYou can now use DriveSync with this account!");
    println!("Your credentials are stored securely in the system keyring.");

    Ok(())
}
