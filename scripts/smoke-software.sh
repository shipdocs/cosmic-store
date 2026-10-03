#!/usr/bin/env bash
# Inspect a local DEB in GNOME Software, before or after installation.
set -euo pipefail
package=$(realpath "$1")
output=$(realpath "$2")
capture=${3:-kompas-in-software}
mkdir -p "$output"
# Start with a separate cache for each observation. Let Software's startup
# refresh settle before asking PackageKit to inspect the local file; competing
# requests can otherwise cancel file-to-app and leave us on the home page.
export XDG_CACHE_HOME
XDG_CACHE_HOME=$(mktemp -d)
gnome-software --quit || true
gnome-software > "$output/software.log" 2>&1 &
software_pid=$!
trap 'kill "$software_pid" 2>/dev/null || true' EXIT
window_id=''
for attempt in $(seq 1 45); do
    kill -0 "$software_pid"
    window_id=$(xdotool search --onlyvisible --pid "$software_pid" 2>/dev/null | tail -1 || true)
    if [ -n "$window_id" ]; then break; fi
    sleep 1
done
test -n "$window_id"
# Open the package only after the initial catalog refresh.
sleep 15
gnome-software --local-filename="$package"
# Software refines local-package metadata asynchronously.
sleep 15
if grep -q 'failed to convert file to GsApp' "$output/software.log"; then
    echo 'Software failed to open the package page.' >&2
    exit 1
fi
import -window "$window_id" "$output/$capture.png"
test -s "$output/$capture.png"
# Record the lower page too, including description and license information.
xdotool mousemove --window "$window_id" 500 400
xdotool click --repeat 20 --delay 50 5
sleep 2
import -window "$window_id" "$output/$capture-details.png"
test -s "$output/$capture-details.png"
