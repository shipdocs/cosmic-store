#!/usr/bin/env bash
# Inspect a local DEB in GNOME Software, before or after installation.
set -euo pipefail
package=$(realpath "$1")
output=$(realpath "$2")
capture=${3:-kompas-in-software}
mkdir -p "$output"
gnome-software --local-filename="$package" > "$output/software.log" 2>&1 &
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
# Software refines local-package metadata asynchronously.
sleep 15
import -window "$window_id" "$output/$capture.png"
test -s "$output/$capture.png"
