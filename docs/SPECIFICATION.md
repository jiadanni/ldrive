# Open Source Linux Google Drive Client - GUI Specification

## Project Overview
**Project Name**: DriveSync for Linux
**Mission**: Free, open source Google Drive client for Linux with native GUI integration
**Target Users**: Linux desktop users who need reliable Google Drive sync without subscription fees
**License**: GPL v3 or MIT (to be decided)

## Core Features

### 1. Authentication & Setup
- **OAuth 2.0 flow** with Google API
- **Multiple account support** with profile switching
- **First-run wizard** for easy configuration
- **Token management** with secure storage (keyring integration)

### 2. File Synchronization
- **Bi-directional sync** with conflict resolution
- **Selective folder sync** with visual folder picker
- **Real-time sync** using Google Drive push notifications
- **Bandwidth management** with throttling controls
- **Delta sync** for efficient large file updates

### 3. User Interface Components

#### Main Application Window
```
┌─────────────────────────────────────────────────────────┐
│ [DriveSync]                    [═] [─] [×]             │
├─────────────────────────────────────────────────────────┤
│ Accounts: [Personal ▼]  [Add Account]    Sync: [Paused] │
├─────────────────────────────────────────────────────────┤
│   [Files]    [Sync Status]    [Settings]    [About]     │
├─────────────────────────────────────────────────────────┤
│ 📁 My Drive                                   [Refresh] │
│ └─ 📁 Documents                    [2.1 GB]  [☑ Synced] │
│ └─ 📁 Photos                       [15.2 GB] [🔄 Syncing]│
│ └─ 📄 report.pdf                   [2.4 MB]  [☑ Synced] │
│ 📁 Shared with me                               [⏸️]    │
│ 📁 Team Drives                                  [⏸️]    │
│                                                    │
│ Storage: 17.3 GB of 15.0 GB used (115%)           │
└─────────────────────────────────────────────────────────┘
```

#### System Tray Integration
- **Sync status icon** (syncing, paused, error)
- **Quick actions menu** (pause/resume, open folder, quit)
- **Progress indicators** for current operations
- **Notification alerts** for conflicts and errors

### 4. File Management
- **Native file manager integration** (Nautilus, Dolphin extensions)
- **Right-click context menus** for quick actions
- **Drag-and-drop support** for uploads
- **File conflict resolution** dialog
- **Offline file access** with smart caching

## Technical Specifications

### Architecture
```
┌─────────────────────────────────────────────────────────┐
│                 GUI Layer (GTK4/Qt6)                    │
├─────────────────────────────────────────────────────────┤
│         Business Logic & Sync Engine (Rust)            │
├─────────────────────────────────────────────────────────┤
│           Google Drive API Client (Rust)               │
├─────────────────────────────────────────────────────────┤
│        System Integration (Linux-specific)             │
└─────────────────────────────────────────────────────────┘
```

### Technology Stack
**Recommended Stack**:
- **Core**: Rust (performance, memory safety)
- **GUI**: GTK4 (native GNOME integration) or Qt6 (KDE compatibility)
- **Bindings**: Rust GTK bindings (gtk-rs) or Slint (Rust-native)
- **Distribution**: Flatpak primary, AppImage secondary
- **Storage**: SQLite for sync state, keyring for tokens

### Dependencies
```toml
# Cargo.toml example
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
google-drive3 = "2.0"  # Google Drive API
serde = { version = "1.0", features = ["derive"] }
rusqlite = "0.29"     # Local database
keyring = "2.0"       # Secure credential storage
notify-rust = "4.0"   # Desktop notifications

# GUI dependencies
gtk = { version = "0.7", package = "gtk4" }
libadwaita = "0.5"    # GNOME HIG components
```

## Implementation Phases

### Phase 1: Core Foundation (MVP)
- [ ] Google Drive API authentication
- [ ] Basic file listing and metadata
- [ ] Simple download/upload operations
- [ ] Minimal GTK4 interface
- [ ] System tray integration

### Phase 2: Full Sync Engine
- [ ] Bi-directional sync with conflict detection
- [ ] Selective folder sync
- [ ] Progress indicators and status reporting
- [ ] File change monitoring (inotify)
- [ ] Error handling and recovery

### Phase 3: Polish & Integration
- [ ] Native file manager extensions
- [ ] Bandwidth management
- [ ] Multiple account support
- [ ] Advanced settings panel
- [ ] Comprehensive logging

### Phase 4: Advanced Features
- [ ] Team Drive support
- [ ] Shared file management
- [ ] Offline document editing
- [ ] Smart sync (cloud-only, local copies)
- [ ] Backup scheduling

## Configuration Management

### Settings Structure
```ini
# ~/.config/drivesync/config.toml
[account]
client_id = "your-oauth-client-id"
client_secret = "your-oauth-secret"  # encrypted
tokens = "encrypted-token-store"

[sync]
root_path = "~/GoogleDrive"
selective_folders = ["Documents", "Photos"]
conflict_resolution = "rename"  # or "overwrite", "ask"

[network]
bandwidth_limit = "0"  # 0 = unlimited
concurrent_uploads = 3
concurrent_downloads = 3

[ui]
start_minimized = false
show_notifications = true
dark_mode = "auto"
```

## System Integration

### Linux Desktop Features
- **AppIndicator** for system tray
- **MIME type** associations
- **File manager actions** (Nautilus/Dolphin)
- **GNOME Online Accounts** integration (optional)
- **DBus services** for inter-process communication

### Distribution Packages
- **Primary**: Flatpak (Flathub)
- **Secondary**: AppImage (portable)
- **Additional**: .deb (Ubuntu/Debian), .rpm (Fedora/openSUSE)
- **AUR package** for Arch Linux

## Performance Targets
- **Memory usage**: < 100MB idle, < 300MB during sync
- **CPU usage**: < 5% during file operations
- **Sync speed**: Limited by network bandwidth
- **Startup time**: < 3 seconds

## Error Handling & Recovery
- **Network failures** with exponential backoff
- **API quota exceeded** with graceful degradation
- **File permission errors** with user notification
- **Corrupted sync state** with rebuild capability
- **Conflict resolution** with user-friendly dialogs

## Security Considerations
- **OAuth tokens** stored in system keyring
- **Minimal permissions** principle
- **Local data encryption** optional
- **Sandboxed execution** via Flatpak
- **No telemetry** or data collection

## Development Setup
```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
flatpak install org.gnome.Sdk org.gnome.Platform

# Build instructions
git clone https://github.com/jiadanni/ldrive
cd ldrive
cargo build --release
cargo run

# Flatpak build
flatpak-builder build com.github.drivesync.yml
flatpak-builder --user --install build com.github.drivesync.yml
```

## Testing Strategy
- **Unit tests** for sync engine and API client
- **Integration tests** with test Google account
- **UI tests** with gtk-rs test utilities
- **Cross-distro testing** via CI/CD
- **Real-world testing** with beta users

## Open Source Considerations
- **Clear contribution guidelines**
- **Code of conduct**
- **Issue templates** for bugs and feature requests
- **Documentation** for users and developers
- **Regular release cycle**

## UI/UX Guidelines

### Design Principles
1. **Native Integration**: Follow GNOME HIG and platform conventions
2. **Clarity**: Clear status indicators and progress feedback
3. **Efficiency**: Minimize clicks for common operations
4. **Safety**: Confirm destructive actions, provide undo where possible
5. **Accessibility**: Keyboard navigation, screen reader support

### Color Coding
- 🟢 **Green**: Synced successfully
- 🔵 **Blue**: Syncing in progress
- 🟡 **Yellow**: Paused or waiting
- 🔴 **Red**: Error or conflict
- ⚪ **Gray**: Disabled or not syncing

### Keyboard Shortcuts
- `Ctrl+N`: Add new account
- `Ctrl+R`: Refresh file list
- `Ctrl+P`: Pause/Resume sync
- `Ctrl+,`: Open settings
- `Ctrl+Q`: Quit application

## API Integration Details

### Google Drive API Scopes Required
```
https://www.googleapis.com/auth/drive.file
https://www.googleapis.com/auth/drive.metadata.readonly
https://www.googleapis.com/auth/drive.readonly
```

### Rate Limiting Strategy
- Implement exponential backoff (2s, 4s, 8s, 16s)
- Batch operations where possible
- Cache metadata to reduce API calls
- Respect quota limits and provide user feedback

### Sync Algorithm
1. **Initial scan**: Build local file tree
2. **Remote scan**: Fetch remote metadata
3. **Diff calculation**: Identify changes
4. **Conflict detection**: Handle concurrent modifications
5. **Operation queue**: Prioritize user-initiated actions
6. **Execute sync**: Upload/download with progress tracking
7. **Update state**: Persist sync state to database

This specification provides a comprehensive foundation for building a modern, native Google Drive client for Linux that respects user freedom while providing a polished experience competitive with proprietary alternatives.
