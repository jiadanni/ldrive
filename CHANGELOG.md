# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Complete Google Drive API integration**:
  - Full OAuth 2.0 authentication flow with PKCE
  - Local server for OAuth callback handling
  - Automatic token refresh functionality
  - Secure credential storage in system keyring
  - Support for multiple accounts

- **Drive API client operations**:
  - List files and folders with pagination
  - Upload files with multipart upload
  - Download files to local storage
  - Update file content
  - Delete files
  - Create folders
  - Get file metadata
  - Track changes with change tokens
  - Get storage quota and account information

- **System tray integration**:
  - StatusNotifierItem protocol support via ksni
  - Dynamic status icons (idle, syncing, paused, error, offline)
  - Context menu with quick actions (pause/resume, open folder, settings, quit)
  - Tooltip with account and storage information
  - Desktop notification manager
  - Support for GNOME, KDE, XFCE, Cinnamon, MATE, and other DEs

- **Desktop notifications**:
  - Sync completion notifications
  - Upload/download complete notifications
  - Error and conflict notifications
  - Storage quota warnings
  - Connection status notifications
  - Customizable notification urgency and timeout

- **Example applications**:
  - `oauth_flow.rs` - Complete OAuth authentication example
  - `list_files.rs` - List Google Drive files
  - `upload_file.rs` - Upload files to Drive
  - `system_tray.rs` - System tray and notification demonstration

- **Documentation**:
  - Comprehensive API Setup Guide
  - Step-by-step Google Cloud Console instructions
  - System Tray Integration Guide
  - Desktop environment compatibility matrix
  - Security best practices
  - Troubleshooting guide
  - Updated README with quick start examples

- **Dependencies**:
  - Added `url` crate for URL parsing
  - Added `base64` crate for encoding
  - Added `ksni` crate for system tray support

### Changed
- Updated Cargo.toml with additional dependencies
- Enhanced README with API setup instructions and examples
- Improved error handling in API client
- Added proper serde field renaming for Google API responses

### Security
- PKCE (Proof Key for Code Exchange) for OAuth 2.0
- Secure credential storage via system keyring
- Automatic token expiration checking and refresh
- No plaintext credential storage

## [0.1.0] - TBD

### Added
- Project initialization
- Core architecture setup
- Development environment

---

**Note**: This project is in early development. The first stable release (v1.0.0) will be announced when core features are complete and thoroughly tested.
