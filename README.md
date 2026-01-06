# COSMIC Store (Wayland Enhanced)

A fork of [COSMIC Store](https://github.com/pop-os/cosmic-store) focusing on Wayland compatibility metadata and search optimizations.

## Features

- **Wayland Compatibility**: Adds parsing and display of Wayland support status (badges, risk levels) from AppStream data.
- **Search Filters**: Additional filtering by download count and Wayland compatibility.
- **Performance**: Async parsing of AppStream data and optimized icon loading.

## Branch Structure

- `master`: Mirrored from upstream (pop-os/cosmic-store).
- `develop`: Active development branch containing all enhancements.

## Build and Run

```bash
git checkout develop
cargo build --release
cargo run --release
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
