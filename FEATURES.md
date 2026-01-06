# COSMIC Store Fork - Features & Branches

## 🌳 Branch Structure

### `master`
- **Status**: Synced with upstream
- **Purpose**: Clean, upstream-compatible version
- **Use case**: Reference for upstream compatibility
- **Latest commit**: Synced with `upstream/master`

### `develop` ⭐ **USE THIS FOR YOUR ENHANCED VERSION**
- **Status**: Your enhanced version with all features
- **Purpose**: Daily development and testing
- **Use case**: Build and run this branch for the full experience
- **Latest commit**: All features integrated

```bash
# To use your enhanced version:
git checkout develop
cargo build --release
```

## ✨ Features in `develop` Branch

### 1. **Wayland Compatibility Detection** 🎯
- Automatic detection of Wayland compatibility for Flatpak apps
- Visual badges (green/orange/red) showing compatibility status
- Framework detection (GTK3/4, Qt5/6, Electron, etc.)
- Risk level assessment (Low/Medium/High/Critical)
- Filter apps by Wayland support

**Commits**: `18abd84`, `8df9030`, `3bdf4e5`, `035cf19`

### 2. **Enhanced Search & Filtering** 🔍
- Filter applications by download count
- Filter by Wayland compatibility status
- Improved AppStream category parsing
- Stats version checking and management

**Commits**: `c7b0a19`, `dd8402c`, `b34d3d9`, `0aa4933`, `0ff2741`, `da06a4f`, `b570f14`, `3bac465`

### 3. **Performance Optimizations** ⚡
- Async stats loading for better responsiveness
- Animated loading screen with oscillating progress bar
- Batch search optimization for explore pages
- Parallel icon loading for search results

**Commits**: `5db5daf`, `b69dacd`, `6b17646`, `3a5f309`

### 4. **Code Refactoring & Modularization** 🏗️
- Modularized codebase with clear separation of concerns
- Extracted pages module (details page, search, etc.)
- Extracted UI components (badges, cards, grid)
- Extracted backend logic (sources, stats, etc.)
- Improved error handling and logging
- Code cleanup and clippy fixes

**Commits**: `68d99bd`, `d5082cc`, `e8fe287`, `9d2c111`, `40319f5`, `4aec141`, `159d459`, `9792bd4`, `195fbe4`, `5bc3f00`, `26ea034`, `c6978c0`, `02422e5`, `ded1b1e`

### 5. **AppStream Optimization** 📦
- Optimized XML parsing for faster app loading
- Improved AppStream cache loading with buffered file reads
- Fixed Wayland badge race condition
- Removed category sanitization for better accuracy

**Commits**: `451b20d`, `a9838f6`, `3d6258c`, `ad7c3fd`

### 6. **Documentation & Cleanup** 📝
- Updated README with fork information
- GitHub Actions workflows for stats generation
- Improved .gitignore for development
- Code quality improvements

**Commits**: `020530f`, `4e8a0e1`, `16bf141`

## 🔄 Workflow

```bash
# Check out the enhanced version
git checkout develop

# Build and run
cargo build --release
cargo run --release

# Run tests
cargo test

# Check code quality
cargo fmt --check
cargo clippy

# When you want to sync with upstream
git fetch upstream
git rebase upstream/master develop
```

## 📊 Statistics

- **Total commits ahead of upstream**: ~30
- **Lines added**: ~6,300
- **Files modified**: 43
- **New modules created**: 6 (pages, ui, backend, etc.)

## 🤝 Contributing

When adding new features:
1. Create a branch from `develop`
2. Make your changes
3. Test thoroughly
4. Merge back to `develop`
5. Keep `master` clean (don't merge to master)

## 📄 License

Same as upstream COSMIC Store project.

