#!/bin/bash

# Usage: ./reboot_phone.sh <UDID>
# Example: ./reboot_phone.sh 0787FFB3-29F1-5D8D-9418-5D3130936709

set -euo pipefail

UDID="${1:-}"
if [[ -z "$UDID" ]]; then
   echo "Usage: $0 <device-udid>" >&2
   exit 1
fi

# Configuration
POLL_INTERVAL=1      # seconds between polls
DOWN_WAIT=15         # max seconds to wait for the device to go down
READY_WAIT=60        # max seconds to wait for the device to come back up

# ------------------------------------------------------------------------------
# Helper: fetch the devicectl device list as JSON from stdout
# ------------------------------------------------------------------------------
fetch_device_json() {
    # xcrun prints on stderr by default, drop it so output doesn't get clobbered
    xcrun devicectl list devices --json-output - 2>/dev/null
}

# ------------------------------------------------------------------------------
# Helper: does devicectl report the device as booted?
# bootState only appears once the device answers enumeration.
# ------------------------------------------------------------------------------
device_reports_booted() {
    local json
    json=$(fetch_device_json) || return 1
    [[ -n "$json" ]] || return 1

    printf '%s' "$json" | jq -e --arg udid "$UDID" '
        .result.devices[]?
        | select(.identifier == $udid)
        | (.deviceProperties.bootState? // "") == "booted"
    ' >/dev/null 2>&1
}

# ------------------------------------------------------------------------------
# Helper: is the device actually ready to use?
# This is an ACTIVE probe: it sends a real command through the CoreDevice tunnel.
# list devices alone cannot distinguish "booted" from "ready" because tunnelState
# and ddiServicesAvailable stay "unavailable"/false even after bootState appears.
# ------------------------------------------------------------------------------
device_is_ready() {
   local out
   out=$(xcrun devicectl device info processes \
             --device "$UDID" --json-output - 2>/dev/null) || return 1

   printf '%s' "$out" | jq -e '.info.outcome == "success"' >/dev/null 2>&1
}

# ------------------------------------------------------------------------------
# 1. Verify the device is currently reachable
# ------------------------------------------------------------------------------
echo "Checking whether device $UDID is reachable..."
if ! device_is_ready; then
   echo "Error: device $UDID is not reachable; cannot reboot it. Might need to be unlocked." >&2
   exit 1
fi
echo "Device $UDID is reachable."

# ------------------------------------------------------------------------------
# 2. Reboot the device
# ------------------------------------------------------------------------------
echo "Rebooting device $UDID..."
xcrun devicectl device reboot --device "$UDID"
echo "Reboot command issued."

# ------------------------------------------------------------------------------
# 3. Wait for the device to actually go down
# ------------------------------------------------------------------------------
echo "Waiting for $UDID to go down..."

down_elapsed=0
while (( down_elapsed < DOWN_WAIT )); do
   if ! device_reports_booted; then
       echo "Device $UDID went down after ${down_elapsed}s."
       break
   fi

   sleep "$POLL_INTERVAL"
   down_elapsed=$((down_elapsed + POLL_INTERVAL))

done

if (( down_elapsed >= DOWN_WAIT )) && device_reports_booted; then
   echo "Warning: device $UDID never reported going down; continuing anyway." >&2
fi

# ------------------------------------------------------------------------------
# 4. Wait for the device to come back and be ready
# ------------------------------------------------------------------------------
echo "Waiting for $UDID to become ready..."
ready_elapsed=0
while (( ready_elapsed < READY_WAIT )); do
   if device_is_ready; then
       echo "Device $UDID is ready. Down detection: ${down_elapsed}s, ready wait: ${ready_elapsed}s, total: $((down_elapsed + ready_elapsed))s."
       exit 0
   fi

   sleep "$POLL_INTERVAL"
   ready_elapsed=$((ready_elapsed + POLL_INTERVAL))
done

echo "Timeout: device $UDID did not become ready within ${READY_WAIT}s." >&2
exit 2


