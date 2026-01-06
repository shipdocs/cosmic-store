# COSMIC Store - Enhanced Fork

An enhanced fork of [COSMIC Store](https://github.com/pop-os/cosmic-store) with additional features for Wayland compatibility and improved search capabilities.

## 🎯 Fork Purpose

This fork adds features specifically designed for Pop!_OS Wayland users:
- **Wayland Compatibility Information** - See which apps work fully on Wayland
- **Enhanced Search Filters** - Filter by download count and Wayland compatibility
- **Performance Optimizations** - Faster app loading and searching
- **Code Improvements** - Better code organization and maintainability

## ✨ Key Features

### Wayland Compatibility Detection
- Automatic detection of Wayland compatibility for applications
- Visual badges showing compatibility status
- Filter apps by Wayland support

### Enhanced Search & Filtering
- Filter applications by download count
- Filter by Wayland compatibility status
- Improved search performance with batch optimization

### Performance Improvements
- Optimized XML parsing for faster app loading
- Async stats loading for better responsiveness
- Batch search optimization for explore pages
- Parallel icon loading for search results

### Code Quality
- Modularized codebase with clear separation of concerns
- Extracted pages, UI components, and backend logic
- Improved error handling and logging
- Comprehensive refactoring for maintainability

## 🔄 Relationship to Upstream

This fork maintains compatibility with the upstream [pop-os/cosmic-store](https://github.com/pop-os/cosmic-store) project. All enhancements are organized in feature branches, making it easy for upstream maintainers to cherry-pick features if desired.

**Master branch:** Stays in sync with upstream for easy merging
**Feature branches:** Contain specific enhancements (see below)

## 📦 Feature Branches

- `feature/wayland-compatibility` - Wayland detection and badges
- `feature/search-filters` - Download count and compatibility filters
- `feature/performance-optimizations` - Performance improvements
- `feature/refactoring` - Code organization and modularization
- `feature/documentation-cleanup` - Documentation improvements

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
