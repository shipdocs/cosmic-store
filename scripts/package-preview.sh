#!/usr/bin/env bash
# Package an already built binary for Zorin 18 / Ubuntu 24.04 testing.
set -euo pipefail
binary=${1:-target/debug/cosmic-store}
output=${2:-dist}
test -x "$binary"
mkdir -p "$output"
output=$(realpath "$output")
architecture=$(dpkg --print-architecture)
version="0.1.0+kompas.$(date -u +%Y%m%d%H%M%S).$(git rev-parse --short HEAD)"
staging=$(mktemp -d)
trap 'rm -rf "$staging"' EXIT
install -Dm0755 "$binary" "$staging/usr/bin/cosmic-store"
strip "$staging/usr/bin/cosmic-store"
install -Dm0644 res/com.system76.CosmicStore.desktop "$staging/usr/share/applications/com.system76.CosmicStore.desktop"
install -Dm0644 res/com.system76.CosmicStore.metainfo.xml "$staging/usr/share/metainfo/com.system76.CosmicStore.metainfo.xml"
while IFS= read -r icon; do
    install -Dm0644 "$icon" "$staging/usr/share/icons/${icon#res/icons/}"
done < <(find res/icons/hicolor -type f -name '*.svg')
install -Dm0644 LICENSE "$staging/usr/share/doc/cosmic-store/copyright"
install -Dm0644 patches/iced_wgpu/LICENSE "$staging/usr/share/doc/cosmic-store/iced-wgpu-license"
install -Dm0644 patches/iced_wgpu/KOMPAS-PATCH.md "$staging/usr/share/doc/cosmic-store/renderer-patch.md"
mkdir -p "$staging/DEBIAN"
# Resolve the actual binary's linked libraries on the target Ubuntu release.
dependencies=$(dpkg-shlibdeps -O -e"$staging/usr/bin/cosmic-store" | sed -n 's/^shlibs:Depends=//p')
test -n "$dependencies"
cat > "$staging/DEBIAN/control" <<EOF
Package: cosmic-store
Version: $version
Architecture: $architecture
Maintainer: ShipDocs <info@shipdocs.app>
Section: admin
Priority: optional
Depends: $dependencies, libxkbcommon-x11-0, apt-config-icons, apt-config-icons-hidpi, apt-config-icons-large, apt-config-icons-large-hidpi
Recommends: flatpak, packagekit, adwaita-icon-theme
Homepage: https://github.com/shipdocs/cosmic-store
Description: Kompas unified software discovery preview
 Apps from configured system and Flatpak sources, plus Steam discovery.
 Preview build for Zorin 18 and Ubuntu 24.04 desktop testing.
EOF
package="$output/kompas_${version}_${architecture}.deb"
dpkg-deb --build --root-owner-group "$staging" "$package"
(cd "$output" && sha256sum "$(basename "$package")" > SHA256SUMS)
printf 'Created %s\n' "$package"
