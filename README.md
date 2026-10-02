# Kompas (COSMIC-based)

An independently maintained fork of [COSMIC Store](https://github.com/pop-os/cosmic-store), targeting Zorin and Ubuntu desktops. It combines Flatpak and PackageKit with software discovery and estimated Wayland compatibility.

The existing `cosmic-store` binary, package name, and application ID are retained for compatibility. This is a ShipDocs project, not an official Zorin store.

## Features

- **Wayland Compatibility**: Shows badges and risk estimates derived from AppStream fields, Flatpak permissions, and framework heuristics. These estimates are not verified compatibility tests.
- **Search Filters**: Sorting by download count, relevance, recent updates, and estimated Wayland compatibility, plus Wayland risk filters.
- **Performance**: Async parsing of AppStream data and optimized icon loading.

## Branch Structure

- `master`: Historical upstream baseline (pop-os/cosmic-store).
- `develop`: Active development branch containing all enhancements.

## Build and Run on Zorin / Ubuntu

Use a current stable Rust toolchain installed through [rustup](https://rustup.rs/).
The manifest requires Rust 1.85 or newer; locked dependencies and workspace tools
may require a newer compiler. The distribution's Rust package may be too old.

Install the native build dependencies:

```bash
sudo apt update
sudo apt install build-essential git pkg-config libflatpak-dev libssl-dev \
    libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libfontconfig1-dev libegl1-mesa-dev
```

Clone and build the development branch without changing the lockfile:

```bash
git clone --branch develop https://github.com/shipdocs/cosmic-store.git
cd cosmic-store
rustup update stable
cargo +stable build --release --locked
cargo +stable run --release --locked
```

Run the store as your normal desktop user, without `sudo`. It uses the system's
configured Flatpak remotes and PackageKit service. For Flatpak apps, ensure
Flatpak and Flathub are available:

```bash
sudo apt install flatpak packagekit
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
```

The default build includes both Flatpak and PackageKit. It uses libcosmic's winit
backend; a COSMIC desktop session is not required by the build instructions.
Actual startup and rendering must still be tested on your Zorin X11/Wayland session.

For startup diagnostics:

```bash
RUST_LOG=cosmic_store=info RUST_BACKTRACE=1 cargo +stable run --release --locked
```

The Debian packaging no longer requires Pop!_OS-specific `appstream-data-pop` or
`cosmic-icons` packages. Packaging and icon rendering still need validation on a
real Zorin installation. The instructions above run directly from the build
directory and do not replace Zorin Software or Bazaar.

A local Debian build uses the existing vendoring recipe and requires `debhelper`
and `just` (at least 1.13; a current release is recommended):

```bash
dpkg-buildpackage -us -uc -b
```

Use this only after the source build and checks succeed. A prebuilt Zorin package
is not supplied yet.

## Development checks

```bash
cargo +stable fmt -- --check
cargo +stable clippy --locked -- -D warnings
cargo +stable test --locked --workspace
```

Pull requests into `develop` and pushes to `develop` run these checks on Ubuntu 24.04.

## Current limitations

- New releases on Steam are now shown; first-added dates for Flatpak/system apps are not available yet.
- Steam discovery and search are integrated. Epic and GOG catalogs remain future work.
- Steam controls purchase and installation; ownership is not checked by this store.
- ProtonDB opens as an external compatibility reference; compatibility ratings are not fetched or asserted.
- Wayland badges are estimates; they do not certify GPU, controller, or runtime compatibility.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Unified discovery

The start page shows new Steam releases and games before general application categories.
Steam artwork, storefront pricing, controller metadata and Linux platform flags come
from Steam's public store endpoints. These endpoints are not a guaranteed stable API.
The Netherlands region is used for displayed prices. Cached featured metadata remains
available if Steam cannot be reached. Local applications remain usable without Steam.

Search first displays results from configured Flatpak and system sources, then adds
Steam matches after a short typing debounce. Plain text queries of at least two
characters are sent to Steam when the Wayland filter is set to All. URI/file/codec
searches remain local. Search GTA expands to Grand Theft Auto; Photoshop, Premiere
and Microsoft Office searches also suggest available native alternatives.

System software is resolved through PackageKit against enabled repositories, including
Zorin's own repositories. Metadata origins no longer need to contain an Ubuntu codename.
The store does not add repositories or expand package permissions automatically.

Steam games have separate actions to open the installation dialog in Steam, view/buy
in the web store, and check ProtonDB. Installation requires a working Steam URI handler
and any required game license. Steam artwork is cached locally; an empty cache displays
a game icon until images arrive. This does not claim that every Windows game or online
mode works on Linux. No Epic/GOG login, purchase or account linking is performed.

### Unified browsing and compatibility

“All apps” lists applications from the configured system/Flatpak catalogs and the fetched Steam featured selection. Live search extends Steam discovery; this is not an exhaustive local index of Steam. The source selector applies to search, category lists and home sections, and can choose a Flatpak alternative when the preferred source is a system package. All result sources share sorting, including Name (A–Z). Popularity and update sorts place items with missing metadata after items with known values; Steam sales are not converted to Flatpak download counts.

Native Linux only is enabled by default. It hides Steam titles unless Steam explicitly reports a native Linux version. Disable it to include titles requiring a Proton compatibility check. PackageKit availability is checked against the configured system; Flatpak catalogs are selected by libflatpak for the host architecture. Native support does not establish that a particular GPU, driver, RAM configuration, anti-cheat setup or desktop session meets an app's requirements. Hardware/Proton compatibility inference remains future work. This conservative default intentionally hides many Windows games that can run well with Proton.

The user-facing product name is **Kompas**, a working name rather than a cleared trademark. The executable and application ID remain `cosmic-store` / `com.system76.CosmicStore` for upgrade compatibility.
