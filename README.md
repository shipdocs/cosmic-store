# COSMIC Store - Enhanced Fork

An enhanced fork of [COSMIC Store](https://github.com/pop-os/cosmic-store) with additional features for Wayland compatibility and improved search capabilities.

## 🎯 Fork Purpose

This fork adds features specifically designed for Pop!_OS Wayland users:
- **Wayland Compatibility Information** - See which apps work fully on Wayland
- **Enhanced Search Filters** - Filter by download count and Wayland compatibility
- **Performance Optimizations** - Faster app loading and searching
- **Code Improvements** - Better code organization and maintainability

## 🌳 Branch Structure

- **`master`** (this branch) - Clean, synced with upstream
- **`develop`** ⭐ - Your enhanced version with all features

**To use the enhanced version:**
```bash
git checkout develop
cargo build --release
```

## ✨ Features

See **[FEATURES.md](FEATURES.md)** for detailed information about all enhancements in the `develop` branch, including:
- Wayland Compatibility Detection
- Enhanced Search & Filtering
- Performance Optimizations
- Code Refactoring & Modularization
- AppStream Optimization
- Documentation & Cleanup

## 🔄 Relationship to Upstream

This fork maintains compatibility with the upstream [pop-os/cosmic-store](https://github.com/pop-os/cosmic-store) project. The `master` branch stays in sync with upstream for easy merging if desired.

## 🚀 Building & Running

### Prerequisites
- Rust 1.70+
- GTK 4.0+
- libadwaita 1.0+

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run --release
```

## 🧪 Testing

```bash
cargo test
```

## 📋 Code Style

This project follows Rust conventions:
- Format code with `cargo fmt`
- Check with `cargo clippy`
- Write descriptive commit messages

## 🤝 Contributing

Contributions are welcome! Please:
1. Create a feature branch from `master`
2. Make your changes
3. Run `cargo fmt` and `cargo clippy`
4. Submit a pull request with a clear description

## 📄 License

Licensed under the same terms as the upstream project (see LICENSE file).

## 🙏 Credits

Built on top of [COSMIC Store](https://github.com/pop-os/cosmic-store) by the Pop!_OS team.
