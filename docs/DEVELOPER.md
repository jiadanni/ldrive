# DriveSync Developer Guide

## Architecture Overview

DriveSync is built with a modular architecture separating concerns:

```
┌─────────────────────────────────────────────────────────┐
│                 GUI Layer (GTK4)                        │
│  - Application window, dialogs, tray icon              │
│  - User interaction handling                           │
├─────────────────────────────────────────────────────────┤
│         Business Logic & Sync Engine                    │
│  - Sync orchestration, conflict resolution             │
│  - File change monitoring and processing               │
├─────────────────────────────────────────────────────────┤
│           Google Drive API Client                       │
│  - OAuth 2.0 authentication                            │
│  - REST API operations                                 │
├─────────────────────────────────────────────────────────┤
│        Storage & System Integration                     │
│  - SQLite database, keyring, file system               │
└─────────────────────────────────────────────────────────┘
```

## Module Breakdown

### `src/api/`
Handles all Google Drive API interactions.

- **`auth.rs`**: OAuth 2.0 flow, token management, keyring storage
- **`client.rs`**: Drive API operations (list, upload, download, delete)
- **`types.rs`**: Data structures for Drive entities

Key design decisions:
- Async/await with tokio runtime
- Automatic token refresh
- Rate limiting with exponential backoff
- Secure credential storage via system keyring

### `src/sync/`
Core synchronization engine.

- **`engine.rs`**: Orchestrates sync operations
- **`monitor.rs`**: File system monitoring (inotify)
- **`resolver.rs`**: Conflict resolution strategies

Sync algorithm:
1. Scan local file system
2. Fetch remote changes from Drive
3. Calculate diff
4. Detect conflicts
5. Execute sync operations
6. Update database state

### `src/storage/`
Local data persistence.

- **`database.rs`**: SQLite schema and operations
- **`sync_state.rs`**: File state tracking

Database schema:
- `files`: File metadata and sync status
- `sync_history`: Audit log of operations
- `conflicts`: Unresolved conflicts

### `src/gui/`
GTK4 user interface.

- Main application window
- Account management dialogs
- Settings panel
- System tray integration

Follows GNOME Human Interface Guidelines (HIG).

### `src/config/`
Application configuration.

- User preferences
- Account settings
- Sync options

Config file: `~/.config/drivesync/config.toml`

## Development Workflow

### Environment Setup

```bash
# Install Rust toolchain
rustup update stable

# Install dev dependencies (Ubuntu/Debian)
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config

# Install dev dependencies (Fedora)
sudo dnf install gtk4-devel libadwaita-devel pkgconfig

# Clone and build
git clone https://github.com/jiadanni/ldrive.git
cd ldrive
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Check without building
cargo check

# Security audit
cargo audit
```

### Debugging

#### Logging

```bash
# Enable debug logs
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=drivesync::sync=trace cargo run

# Log to file
RUST_LOG=debug cargo run 2>&1 | tee drivesync.log
```

#### GTK Inspector

```bash
# Enable GTK inspector
GTK_DEBUG=interactive cargo run
```

#### Database Inspection

```bash
# Open database
sqlite3 ~/.local/share/drivesync/drivesync.db

# Check schema
.schema

# Query files
SELECT * FROM files;
```

## Adding Features

### Implementing a New Sync Strategy

1. **Define strategy** in `src/sync/mod.rs`:
   ```rust
   pub enum SyncStrategy {
       TwoWay,
       UploadOnly,
       DownloadOnly,
   }
   ```

2. **Implement logic** in `src/sync/engine.rs`:
   ```rust
   impl SyncEngine {
       async fn execute_strategy(&self, strategy: SyncStrategy) {
           // Implementation
       }
   }
   ```

3. **Add UI controls** in `src/gui/settings.rs`

4. **Update config** in `src/config/mod.rs`

5. **Write tests** in `tests/sync_strategies.rs`

### Adding API Endpoints

1. **Define types** in `src/api/types.rs`
2. **Implement method** in `src/api/client.rs`
3. **Handle errors** appropriately
4. **Add tests** with mocked responses

### Adding UI Components

1. **Follow GNOME HIG**
2. **Use libadwaita widgets**
3. **Handle async operations** with `glib::MainContext`
4. **Update translations**

## Testing Strategy

### Unit Tests

Located alongside source code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test implementation
    }
}
```

### Integration Tests

Located in `tests/`:

```rust
// tests/sync_integration.rs
#[tokio::test]
async fn test_full_sync() {
    // Setup
    // Execute
    // Assert
}
```

### Mocking

Use `mockall` for mocking:

```rust
use mockall::mock;

mock! {
    DriveClient {
        async fn list_files(&self) -> Result<Vec<DriveFile>>;
    }
}
```

## Performance Optimization

### Profiling

```bash
# CPU profiling
cargo build --release
perf record target/release/drivesync
perf report

# Memory profiling
valgrind --tool=massif target/release/drivesync
```

### Benchmarking

```bash
# Run benchmarks
cargo bench
```

### Optimization Tips

1. **Use async** for I/O operations
2. **Batch API calls** where possible
3. **Cache metadata** to reduce API calls
4. **Use database indexes** for queries
5. **Stream large files** instead of loading into memory

## Release Process

### Versioning

Follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Update `Cargo.toml` version

### Building Release

```bash
# Build optimized binary
cargo build --release

# Strip debug symbols
strip target/release/drivesync

# Create tarball
tar czf drivesync-$VERSION.tar.gz -C target/release drivesync
```

### Creating Packages

#### Flatpak

```bash
flatpak-builder --repo=repo build com.github.drivesync.yml
flatpak build-bundle repo drivesync.flatpak com.github.drivesync
```

#### AppImage

```bash
# Use appimagetool
appimagetool drivesync.AppDir/
```

#### Debian Package

```bash
cargo deb
```

### Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update CHANGELOG.md
- [ ] Run full test suite
- [ ] Test on multiple distros
- [ ] Build all packages
- [ ] Create GitHub release
- [ ] Update documentation
- [ ] Announce release

## Common Issues

### Build Failures

**GTK headers not found**:
```bash
sudo apt install libgtk-4-dev
```

**Linker errors**:
```bash
sudo apt install pkg-config
```

### Runtime Issues

**Keyring not available**:
- Ensure gnome-keyring or equivalent is running
- Check `secret-tool` works

**File monitoring not working**:
- Check inotify limits: `/proc/sys/fs/inotify/max_user_watches`
- Increase if needed: `sudo sysctl fs.inotify.max_user_watches=524288`

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.

### Code Review Checklist

- [ ] Code follows Rust style guidelines
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Commit messages are clear
- [ ] No hardcoded credentials
- [ ] Error handling is appropriate
- [ ] Performance impact considered

## Resources

### Rust Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)

### GTK Resources
- [GTK Documentation](https://docs.gtk.org/)
- [gtk-rs Book](https://gtk-rs.org/gtk4-rs/stable/latest/book/)
- [GNOME HIG](https://developer.gnome.org/hig/)

### Google Drive API
- [Drive API v3 Reference](https://developers.google.com/drive/api/v3/reference)
- [OAuth 2.0](https://developers.google.com/identity/protocols/oauth2)

## Contact

- GitHub: https://github.com/jiadanni/ldrive
- Issues: https://github.com/jiadanni/ldrive/issues
- Discussions: https://github.com/jiadanni/ldrive/discussions

Happy coding! 🦀
