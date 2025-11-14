/// OAuth 2.0 authentication management
///
/// Handles the OAuth flow for Google Drive API access, including:
/// - Initial authentication with browser-based flow
/// - Token storage in system keyring
/// - Token refresh when expired
/// - Multiple account support

use crate::api::{ApiError, Result};
use chrono::{DateTime, Duration, Utc};
use keyring::Entry;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken, Scope, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

// Google OAuth endpoints
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

// Required scopes for Google Drive access
const DRIVE_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/drive.file",
    "https://www.googleapis.com/auth/drive.metadata.readonly",
    "https://www.googleapis.com/auth/userinfo.email",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub email: String,
}

pub struct AuthManager {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl AuthManager {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri: "http://localhost:8080".to_string(),
        }
    }

    /// Create OAuth client
    fn create_client(&self) -> Result<BasicClient> {
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(GOOGLE_AUTH_URL.to_string())
                .map_err(|e| ApiError::AuthenticationError(format!("Invalid auth URL: {}", e)))?,
            Some(
                TokenUrl::new(GOOGLE_TOKEN_URL.to_string()).map_err(|e| {
                    ApiError::AuthenticationError(format!("Invalid token URL: {}", e))
                })?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(self.redirect_uri.clone())
                .map_err(|e| ApiError::AuthenticationError(format!("Invalid redirect URI: {}", e)))?,
        );

        Ok(client)
    }

    /// Start OAuth flow and return authorization URL
    pub fn get_auth_url(&self) -> Result<(String, CsrfToken, PkceCodeChallenge)> {
        let client = self.create_client()?;

        // Generate PKCE challenge for added security
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(DRIVE_SCOPES.iter().map(|s| Scope::new(s.to_string())))
            .set_pkce_challenge(pkce_challenge.clone())
            .add_extra_param("access_type", "offline")
            .add_extra_param("prompt", "consent")
            .url();

        Ok((auth_url.to_string(), csrf_token, pkce_challenge))
    }

    /// Start local server to listen for OAuth callback
    pub async fn listen_for_callback(&self) -> Result<String> {
        let listener = TcpListener::bind("127.0.0.1:8080")
            .map_err(|e| ApiError::AuthenticationError(format!("Failed to bind to port: {}", e)))?;

        tracing::info!("Listening for OAuth callback on http://localhost:8080");

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut reader = BufReader::new(&stream);
                let mut request_line = String::new();
                reader
                    .read_line(&mut request_line)
                    .map_err(|e| ApiError::AuthenticationError(format!("Failed to read request: {}", e)))?;

                let redirect_url = request_line.split_whitespace().nth(1).ok_or_else(|| {
                    ApiError::AuthenticationError("Invalid request format".to_string())
                })?;

                let url = Url::parse(&format!("http://localhost{}", redirect_url))
                    .map_err(|e| ApiError::AuthenticationError(format!("Invalid URL: {}", e)))?;

                // Send success response to browser
                let response = "HTTP/1.1 200 OK\r\n\r\n\
                    <html><body>\
                    <h1>Authentication Successful!</h1>\
                    <p>You can close this window and return to DriveSync.</p>\
                    </body></html>";
                stream
                    .write_all(response.as_bytes())
                    .map_err(|e| ApiError::AuthenticationError(format!("Failed to send response: {}", e)))?;

                // Extract authorization code
                let code = url
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, code)| code.to_string())
                    .ok_or_else(|| ApiError::AuthenticationError("No authorization code found".to_string()))?;

                return Ok(code);
            }
        }

        Err(ApiError::AuthenticationError(
            "Failed to receive callback".to_string(),
        ))
    }

    /// Exchange authorization code for access tokens
    pub async fn exchange_code(&self, code: &str, pkce_verifier: PkceCodeChallenge) -> Result<Credentials> {
        let client = self.create_client()?;

        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(oauth2::PkceCodeVerifier::new(pkce_verifier.as_str().to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| ApiError::AuthenticationError(format!("Token exchange failed: {}", e)))?;

        let access_token = token_result.access_token().secret().to_string();
        let refresh_token = token_result
            .refresh_token()
            .ok_or_else(|| ApiError::AuthenticationError("No refresh token received".to_string()))?
            .secret()
            .to_string();

        let expires_at = Utc::now()
            + Duration::seconds(
                token_result
                    .expires_in()
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(3600),
            );

        // Get user email
        let email = self.get_user_email(&access_token).await?;

        Ok(Credentials {
            access_token,
            refresh_token,
            expires_at,
            email,
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, credentials: &Credentials) -> Result<Credentials> {
        let client = self.create_client()?;

        let token_result = client
            .exchange_refresh_token(&RefreshToken::new(credentials.refresh_token.clone()))
            .request_async(async_http_client)
            .await
            .map_err(|e| ApiError::AuthenticationError(format!("Token refresh failed: {}", e)))?;

        let access_token = token_result.access_token().secret().to_string();

        let expires_at = Utc::now()
            + Duration::seconds(
                token_result
                    .expires_in()
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(3600),
            );

        Ok(Credentials {
            access_token,
            refresh_token: credentials.refresh_token.clone(),
            expires_at,
            email: credentials.email.clone(),
        })
    }

    /// Get user email from access token
    async fn get_user_email(&self, access_token: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e))?;

        if !response.status().is_success() {
            return Err(ApiError::AuthenticationError(format!(
                "Failed to get user info: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct UserInfo {
            email: String,
        }

        let user_info: UserInfo = response
            .json()
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Invalid user info response: {}", e)))?;

        Ok(user_info.email)
    }

    /// Check if credentials are expired
    pub fn is_expired(&self, credentials: &Credentials) -> bool {
        Utc::now() >= credentials.expires_at
    }

    /// Get valid credentials, refreshing if necessary
    pub async fn get_valid_credentials(&self, account_email: &str) -> Result<Credentials> {
        let credentials = self.get_credentials(account_email)?;

        if self.is_expired(&credentials) {
            tracing::info!("Access token expired, refreshing...");
            let new_credentials = self.refresh_token(&credentials).await?;
            self.store_credentials(account_email, &new_credentials)?;
            Ok(new_credentials)
        } else {
            Ok(credentials)
        }
    }

    /// Store credentials in system keyring
    pub fn store_credentials(&self, account_email: &str, credentials: &Credentials) -> Result<()> {
        let entry = Entry::new("drivesync", account_email)
            .map_err(|e| ApiError::AuthenticationError(format!("Keyring error: {}", e)))?;

        let creds_json = serde_json::to_string(credentials)
            .map_err(|e| ApiError::InvalidResponse(format!("JSON serialization error: {}", e)))?;

        entry
            .set_password(&creds_json)
            .map_err(|e| ApiError::AuthenticationError(format!("Failed to store credentials: {}", e)))?;

        Ok(())
    }

    /// Retrieve credentials from system keyring
    pub fn get_credentials(&self, account_email: &str) -> Result<Credentials> {
        let entry = Entry::new("drivesync", account_email)
            .map_err(|e| ApiError::AuthenticationError(format!("Keyring error: {}", e)))?;

        let creds_json = entry
            .get_password()
            .map_err(|e| ApiError::AuthenticationError(format!("Failed to retrieve credentials: {}", e)))?;

        let credentials = serde_json::from_str(&creds_json)
            .map_err(|e| ApiError::InvalidResponse(format!("JSON deserialization error: {}", e)))?;

        Ok(credentials)
    }

    /// Delete credentials from system keyring
    pub fn delete_credentials(&self, account_email: &str) -> Result<()> {
        let entry = Entry::new("drivesync", account_email)
            .map_err(|e| ApiError::AuthenticationError(format!("Keyring error: {}", e)))?;

        entry
            .delete_credential()
            .map_err(|e| ApiError::AuthenticationError(format!("Failed to delete credentials: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager_creation() {
        let auth = AuthManager::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        assert_eq!(auth.client_id, "test_client_id");
    }
}
