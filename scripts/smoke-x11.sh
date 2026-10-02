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
    window_id=$(xwininfo -root -tree | awk '/Kompas/ { print $1; exit }')
    if [ -n "$window_id" ]; then break; fi
    sleep 1
done
if [ -z "$window_id" ]; then
    cat "$output/startup.log"
    echo 'Kompas did not create an X11 window.' >&2
    exit 1
fi
# Give background storefront discovery time to populate the first rows.
sleep 20
kill -0 "$app_pid"
import -window "$window_id" "$output/store-x11.png"
test -s "$output/store-x11.png"

# Browse the unified catalog, using the stable second navigation entry.
xdotool mousemove --window "$window_id" 90 112 click 1
for attempt in $(seq 1 60); do
    if grep -Eq 'searched for categories \[\].*found [1-9][0-9]* results' "$output/startup.log"; then break; fi
    kill -0 "$app_pid"
    sleep 1
done
grep -Eq 'searched for categories \[\].*found [1-9][0-9]* results' "$output/startup.log"
# Trigger a fresh draw after background results replace the previous page.
xdotool mousemove --window "$window_id" 500 220
sleep 2
kill -0 "$app_pid"
import -window "$window_id" "$output/all-apps-native.png"
test -s "$output/all-apps-native.png"

# Exercise actual keyboard search and its recoverable empty state.
xdotool windowfocus "$window_id"
xdotool key --window "$window_id" ctrl+f
xdotool type --window "$window_id" --delay 100 'gimp'
for attempt in $(seq 1 20); do
    if grep -Eq 'search "gimp" ready: [1-9][0-9]* results' "$output/startup.log"; then break; fi
    kill -0 "$app_pid"
    sleep 1
done
grep -Eq 'search "gimp" ready: [1-9][0-9]* results' "$output/startup.log"
sleep 2
import -window "$window_id" "$output/search-gimp.png"
xdotool key --window "$window_id" ctrl+a
xdotool type --window "$window_id" --delay 40 'kompas-no-such-app-987654321'
sleep 3
kill -0 "$app_pid"
import -window "$window_id" "$output/search-empty.png"
# Narrow window exercises the stacked filter layout.
xdotool windowsize "$window_id" 620 768
sleep 2
import -window "$window_id" "$output/narrow.png"
