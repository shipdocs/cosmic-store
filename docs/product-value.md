# Kompas: value and acceptance criteria

Kompas is the working product name. It is independent of Zorin branding and its maintainer's business name. The name has not undergone trademark clearance. The repository, executable and app ID remain stable during development.

## Existing alternatives

- [Bazaar](https://apps.gnome.org/Bazaar/) already provides a modern Flatpak-focused store, app permissions and Flathub favorites.
- [KDE Discover](https://apps.kde.org/discover/) already combines system packages, Flatpak, Snap and some AppImages.

Combining system packages and Flatpak alone is therefore not a differentiator. A modern layout is also insufficient justification for maintaining another store.

## What this implementation adds

- Validated Steam product types exclude hardware/video that storefront feeds also call apps.
- Current Steam games and live storefront searches beside configured system/Flatpak applications, with explicit install, purchase and compatibility handoffs.
- One source filter and shared sorting for browsing, categories and search. An All apps page covers the loaded catalogs; Steam's complete catalog is accessible through live search rather than fully indexed locally.
- Native Linux filtering enabled by default; Windows games can be included deliberately. Steam discovery is currently limited to x86_64 hosts, matching the supported Linux client target. Native availability is not a hardware guarantee.
- PackageKit availability checks rather than Ubuntu-origin assumptions, retaining Zorin repository applications and usable source alternatives.
- Suggestions for available Linux alternatives to a few common Windows programs.

These are implemented capabilities, not evidence that users find this store better than existing alternatives. No game purchases or system installations are performed by development tests.

## Remaining gap

Kompas is an initial unified discovery layer. Epic/GOG, automatically established Proton compatibility, anti-cheat status, GPU/driver/RAM requirements and system-specific compatibility explanations are not implemented. Flathub popularity and update data are not comparable to Steam sales or release data. Missing metadata must not be presented as a measured score.

## Practical acceptance on Zorin

1. Search for Spotify, choose a configured source and install successfully without needing to understand package formats.
2. Search for GTA, include games requiring Proton, open the correct Steam page and assess its compatibility without the store promising it will run.
3. Browse All apps and Games, filter by system/Flatpak/Steam and Name, and restore the complete available result set by removing filters.
4. Confirm an unavailable system package is hidden while an available Flatpak alternative remains visible, including Zorin-specific origins.
5. Demonstrate that supported architecture/availability rules hide unusable options without hiding valid installed apps.

CI validates parsing, source filtering, availability rules, sorting and X11 startup. The end-to-end installation/launch criteria need a real desktop session. Sustained value should be judged by whether these tasks require fewer source choices and failed installs than Bazaar/Discover, rather than by screenshots alone.
