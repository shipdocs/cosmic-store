#!/usr/bin/env bash
# Run inside an X11 session (CI uses dbus-run-session and xvfb-run).
set -euo pipefail
binary=${1:-target/debug/cosmic-store}
output=${2:-smoke-artifacts}
mkdir -p "$output"
RUST_LOG=cosmic_store=info "$binary" > "$output/startup.log" 2>&1 &
app_pid=$!
trap 'kill "$app_pid" 2>/dev/null || true' EXIT
window_id=''
for attempt in $(seq 1 25); do
    if ! kill -0 "$app_pid" 2>/dev/null; then
        cat "$output/startup.log"
        exit 1
    fi
    window_id=$(xwininfo -root -tree | awk '/ShipDocs Store/ { print $1; exit }')
    if [ -n "$window_id" ]; then break; fi
    sleep 1
done
if [ -z "$window_id" ]; then
    cat "$output/startup.log"
    echo 'ShipDocs Store did not create an X11 window.' >&2
    exit 1
fi
# Give background storefront discovery time to populate the first rows.
sleep 12
kill -0 "$app_pid"
import -window "$window_id" "$output/store-x11.png"
test -s "$output/store-x11.png"
