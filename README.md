# DriveSync for Linux

<div align="center">

![DriveSync Logo](docs/logo.png)

**Free, Open Source Google Drive Client for Linux**

[![License: MIT OR GPL-3.0](https://img.shields.io/badge/License-MIT%20OR%20GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![GTK](https://img.shields.io/badge/GTK-4.12%2B-green.svg)](https://www.gtk.org/)

</div>

## 🎯 Mission

Provide Linux desktop users with a reliable, native Google Drive sync client without subscription fees. DriveSync respects user freedom while delivering a polished experience competitive with proprietary alternatives.

## ✨ Features

### Current Status: 🚧 Early Development (v0.1.0)

- **🔐 OAuth 2.0 Authentication** - Secure Google account integration
- **📁 Bi-directional Sync** - Keep your files synchronized automatically
- **🎨 Native GTK4 Interface** - Beautiful, modern Linux desktop integration
- **🔔 System Tray Integration** - Quick access and status monitoring
- **⚙️ Selective Folder Sync** - Choose which folders to sync
- **💾 Smart Caching** - Efficient storage and bandwidth usage

### Planned Features

- **👥 Multiple Account Support** - Manage multiple Google Drive accounts
- **🚀 Real-time Sync** - Push notifications for instant updates
- **🎮 Bandwidth Management** - Control upload/download speeds
- **📊 Team Drive Support** - Collaborate with team drives
- **🔄 Conflict Resolution** - Smart handling of file conflicts
- **📦 File Manager Integration** - Nautilus, Dolphin extensions

## 🖼️ Screenshots

*Coming soon*

## 🚀 Quick Start

### Installation

#### From Flatpak (Recommended)
```bash
# Coming soon to Flathub
flatpak install flathub com.github.drivesync
```

#### From AppImage
```bash
# Download latest release
wget https://github.com/jiadanni/ldrive/releases/latest/download/drivesync.AppImage
chmod +x drivesync.AppImage
./drivesync.AppImage
```

#### From Source
```bash
# Prerequisites: Rust 1.70+, GTK4 development libraries
sudo apt install libgtk-4-dev libadwaita-1-dev # Ubuntu/Debian
# or
sudo dnf install gtk4-devel libadwaita-devel # Fedora

# Build and install
git clone https://github.com/jiadanni/ldrive.git
cd ldrive
cargo build --release
sudo cp target/release/drivesync /usr/local/bin/
```

### Setup

1. Launch DriveSync from your application menu
2. Click "Add Account" and sign in with your Google account
3. Select which folders to sync
4. Choose your local sync directory
5. Start syncing!

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                 GUI Layer (GTK4/libadwaita)             │
│              - Main window, tray icon, dialogs          │
├─────────────────────────────────────────────────────────┤
│         Business Logic & Sync Engine (Rust)             │
│    - Conflict resolution, file monitoring, scheduling   │
├─────────────────────────────────────────────────────────┤
│           Google Drive API Client (Rust)                │
│       - OAuth, file operations, metadata sync           │
├─────────────────────────────────────────────────────────┤
│        System Integration (Linux-specific)              │
│   - File notifications, keyring, DBus, file manager     │
└─────────────────────────────────────────────────────────┘
```

## 🛠️ Development

### Prerequisites

- Rust 1.70 or later
- GTK 4.12 or later
- libadwaita 1.4 or later
- SQLite 3

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure

```
ldrive/
├── src/
│   ├── main.rs              # Application entry point
│   ├── api/                 # Google Drive API client
│   ├── sync/                # Synchronization engine
│   ├── gui/                 # GTK4 user interface
│   ├── storage/             # Local database and cache
│   └── config/              # Configuration management
├── tests/                   # Integration tests
├── docs/                    # Documentation
└── examples/                # Example code
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Areas We Need Help

- 🎨 UI/UX design and improvements
- 🐛 Bug reports and testing
- 📝 Documentation
- 🌍 Translations
- 🔧 File manager extensions

## 📋 Roadmap

### Phase 1: Core Foundation (Current)
- [x] Project setup and architecture
- [ ] Google Drive API authentication
- [ ] Basic file listing and metadata
- [ ] Simple download/upload operations
- [ ] Minimal GTK4 interface

### Phase 2: Full Sync Engine
- [ ] Bi-directional sync with conflict detection
- [ ] Selective folder sync
- [ ] Progress indicators
- [ ] File change monitoring
- [ ] Error handling and recovery

### Phase 3: Polish & Integration
- [ ] Native file manager extensions
- [ ] Bandwidth management
- [ ] Multiple account support
- [ ] Advanced settings
- [ ] Comprehensive logging

### Phase 4: Advanced Features
- [ ] Team Drive support
- [ ] Shared file management
- [ ] Offline document editing
- [ ] Smart sync modes
- [ ] Backup scheduling

## 📖 Documentation

- [GUI Specification](docs/SPECIFICATION.md) - Detailed design document
- [API Documentation](https://docs.rs/drivesync) - Code documentation
- [User Guide](docs/USER_GUIDE.md) - End-user documentation
- [Developer Guide](docs/DEVELOPER.md) - Contributor documentation

## 🔒 Security

- OAuth tokens stored securely in system keyring
- Minimal permission principle
- No telemetry or data collection
- Optional local data encryption
- Sandboxed execution via Flatpak

## 📜 License

This project is dual-licensed under:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- GNU General Public License v3.0 ([LICENSE-GPL](LICENSE-GPL))

You may choose either license for your use.

## 🙏 Acknowledgments

- Google Drive API team for excellent documentation
- GTK and GNOME communities for amazing tools
- Rust community for the incredible ecosystem
- All contributors and testers

## 📞 Support

- 🐛 [Issue Tracker](https://github.com/jiadanni/ldrive/issues)
- 💬 [Discussions](https://github.com/jiadanni/ldrive/discussions)
- 📧 Email: support@drivesync.dev

## ⭐ Star History

If you find this project useful, please consider giving it a star!

---

<div align="center">
Made with ❤️ by the DriveSync community
</div>
