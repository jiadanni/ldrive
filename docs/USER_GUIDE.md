# DriveSync User Guide

## Table of Contents
1. [Installation](#installation)
2. [Getting Started](#getting-started)
3. [Managing Accounts](#managing-accounts)
4. [Synchronization](#synchronization)
5. [Settings](#settings)
6. [Troubleshooting](#troubleshooting)

## Installation

### Flatpak (Recommended)

```bash
flatpak install flathub com.github.drivesync
```

### AppImage

1. Download the latest AppImage from [releases](https://github.com/jiadanni/ldrive/releases)
2. Make it executable: `chmod +x drivesync.AppImage`
3. Run it: `./drivesync.AppImage`

### From Source

See [README.md](../README.md#from-source) for build instructions.

## Getting Started

### First Launch

1. Launch DriveSync from your application menu
2. Click "Add Google Account"
3. Your browser will open for Google authentication
4. Sign in and authorize DriveSync
5. Choose your local sync folder
6. Select which Drive folders to sync

### System Tray

DriveSync runs in your system tray. Click the icon to:
- View sync status
- Pause/resume synchronization
- Open sync folder
- Access settings
- Quit application

## Managing Accounts

### Adding an Account

1. Click the tray icon
2. Select "Add Account"
3. Follow the authentication process
4. Configure sync preferences

### Switching Accounts

1. Open the main window
2. Click the account dropdown
3. Select the account to manage

### Removing an Account

1. Open Settings
2. Go to Accounts tab
3. Select account and click "Remove"
4. Confirm deletion

**Note**: This only removes the account from DriveSync, not from Google.

## Synchronization

### Automatic Sync

By default, DriveSync automatically syncs changes:
- **Local changes** are uploaded to Google Drive
- **Remote changes** are downloaded to your computer
- **Monitoring** happens in real-time

### Manual Sync

Force a sync by:
- Clicking "Sync Now" in the tray menu
- Right-clicking a file and selecting "Sync"

### Selective Sync

Choose which folders to sync:

1. Open Settings → Sync
2. Click "Select Folders"
3. Check/uncheck folders
4. Click "Apply"

**Note**: Unselecting a folder removes it from your local drive.

### Conflict Resolution

When a file is modified both locally and remotely:

- **Rename** (default): Both versions are kept
- **Ask**: You choose which version to keep
- **Overwrite**: Remote version always wins

Configure in Settings → Sync → Conflict Resolution

## Settings

### General

- **Start on login**: Launch DriveSync at startup
- **Start minimized**: Hide window on launch
- **Show notifications**: Display sync notifications

### Sync

- **Sync folder location**: Where files are stored
- **Selective folders**: Choose which folders to sync
- **Conflict resolution**: How to handle conflicts
- **Sync hidden files**: Include dotfiles

### Network

- **Bandwidth limit**: Throttle upload/download speed
- **Concurrent uploads**: Number of parallel uploads
- **Concurrent downloads**: Number of parallel downloads

### Appearance

- **Theme**: Auto, Light, or Dark
- **Tray icon style**: Choose icon appearance

## Troubleshooting

### Sync Not Working

1. Check internet connection
2. Verify account authentication: Settings → Accounts
3. Check available storage space
4. Review logs: `~/.local/share/drivesync/drivesync.log`

### Authentication Issues

1. Remove and re-add account
2. Check system keyring is working
3. Clear credentials: `secret-tool clear account YOUR_EMAIL`

### High CPU/Memory Usage

1. Reduce concurrent operations in Settings → Network
2. Exclude large folders from sync
3. Check for file monitoring issues on large directories

### File Conflicts

1. Open the Conflicts tab
2. Review conflicting files
3. Choose resolution for each
4. Click "Resolve All" or resolve individually

### Logs and Debugging

Enable debug logging:

```bash
RUST_LOG=debug drivesync
```

Logs location: `~/.local/share/drivesync/drivesync.log`

### Getting Help

- [GitHub Issues](https://github.com/jiadanni/ldrive/issues)
- [Discussions](https://github.com/jiadanni/ldrive/discussions)
- Check FAQ in docs

## Advanced Usage

### Command Line

```bash
# Start with specific account
drivesync --account user@example.com

# Sync specific folder
drivesync --sync-path /path/to/folder

# One-time sync (don't stay running)
drivesync --sync-once

# Debug mode
drivesync --debug
```

### Configuration File

Location: `~/.config/drivesync/config.toml`

Edit manually for advanced settings (not recommended for beginners).

## Tips and Best Practices

1. **Regular backups**: DriveSync is not a backup solution
2. **Storage space**: Keep 10-20% free space on both local and Drive
3. **Large files**: Consider selective sync for large folders
4. **Bandwidth**: Set limits if on metered connection
5. **Conflicts**: Address conflicts promptly

## Privacy and Security

- Credentials stored securely in system keyring
- No telemetry or data collection
- Local database encrypted (optional)
- Open source - audit the code!

---

For more information, see [SPECIFICATION.md](SPECIFICATION.md) or visit the [project website](https://github.com/jiadanni/ldrive).
