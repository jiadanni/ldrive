# Contributing to DriveSync for Linux

Thank you for your interest in contributing to DriveSync! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful and constructive in all interactions.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the issue
- **Expected behavior** vs actual behavior
- **System information** (OS, version, desktop environment)
- **Relevant logs** (use `RUST_LOG=debug` for detailed logs)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:

- **Clear description** of the proposed feature
- **Use cases** and motivation
- **Possible implementation** approach (if you have ideas)
- **Alternatives considered**

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following our coding standards
3. **Add tests** if applicable
4. **Update documentation** if needed
5. **Ensure tests pass**: `cargo test`
6. **Run formatter**: `cargo fmt`
7. **Run linter**: `cargo clippy`
8. **Commit with clear messages**
9. **Submit pull request**

#### Commit Message Guidelines

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Example:
```
feat(sync): Add selective folder sync support

- Implement folder selection UI
- Add database schema for selective sync
- Update sync engine to respect folder preferences

Closes #123
```

## Development Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install GTK4 development libraries
# Ubuntu/Debian:
sudo apt install libgtk-4-dev libadwaita-1-dev

# Fedora:
sudo dnf install gtk4-devel libadwaita-devel

# Arch:
sudo pacman -S gtk4 libadwaita
```

### Building and Testing

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/ldrive.git
cd ldrive

# Build
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

## Project Structure

```
ldrive/
├── src/
│   ├── main.rs           # Application entry point
│   ├── api/              # Google Drive API client
│   │   ├── auth.rs       # OAuth authentication
│   │   ├── client.rs     # API operations
│   │   └── types.rs      # Data types
│   ├── sync/             # Sync engine
│   │   ├── engine.rs     # Core sync logic
│   │   ├── monitor.rs    # File monitoring
│   │   └── resolver.rs   # Conflict resolution
│   ├── gui/              # GTK4 interface
│   ├── storage/          # Database and caching
│   └── config/           # Configuration
├── tests/                # Integration tests
└── docs/                 # Documentation
```

## Coding Standards

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` with default settings
- Address all `clippy` warnings
- Write documentation for public APIs
- Include examples in doc comments

### Error Handling

- Use custom error types with `thiserror`
- Provide context with `anyhow` where appropriate
- Log errors appropriately
- Return `Result` types for fallible operations

### Testing

- Write unit tests for individual functions
- Add integration tests for features
- Mock external dependencies
- Aim for >80% code coverage

### Documentation

- Document all public APIs
- Include examples in doc comments
- Update user-facing docs for new features
- Keep README.md up to date

## Areas We Need Help

### High Priority

- 🔐 OAuth 2.0 implementation
- 📁 File synchronization engine
- 🎨 UI/UX improvements
- 🧪 Test coverage

### Medium Priority

- 🐋 File manager extensions
- 🌍 Internationalization (i18n)
- 📦 Packaging (Flatpak, AppImage)
- 📚 Documentation

### Good First Issues

Check issues labeled `good first issue` for beginner-friendly tasks.

## Review Process

1. **Automated checks** must pass (tests, lint, format)
2. **Code review** by maintainer(s)
3. **Testing** on multiple distros if applicable
4. **Documentation** review
5. **Merge** after approval

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and GPL-3.0.

## Questions?

- Open a [GitHub Discussion](https://github.com/jiadanni/ldrive/discussions)
- Check existing issues and documentation
- Reach out to maintainers

Thank you for contributing to DriveSync! 🚀
