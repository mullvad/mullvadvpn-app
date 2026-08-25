#!/usr/bin/env bash

# Authenticode signs the given Windows binaries with the certificate whose thumbprint is in
# $CERT_HASH.
#
# Usage: sign-windows.sh [--description <description>] <binary>...

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# shellcheck source=scripts/utils/log
source "$SCRIPT_DIR/utils/log"

# The description embedded in the signature, shown in the UAC prompt among other places.
DESCRIPTION="Mullvad VPN"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --description)
            DESCRIPTION="$2"
            shift
            ;;
        --) shift; break;;
        -*)
            log_error "Unknown parameter: $1"
            exit 1
            ;;
        *) break;;
    esac
    shift
done

if [[ "$#" -eq 0 ]]; then
    log_error "No binaries to sign were given"
    exit 1
fi

if [[ -z ${CERT_HASH-} ]]; then
    log_error "The variable CERT_HASH is not set. It needs to be set to the thumbprint of"
    log_error "the signing certificate."
    exit 1
fi

NUM_RETRIES=3

for binary in "$@"; do
    # Some binaries, such as those shipped by third parties, are already signed.
    if signtool verify -pa -q "$binary" > /dev/null 2>&1; then
        log_info "Skipping already signed $binary"
        continue
    fi

    # Try multiple times in case the timestamp server cannot be contacted.
    for i in $(seq 0 ${NUM_RETRIES}); do
        log_info "Signing $binary..."
        if signtool sign \
            -tr http://timestamp.digicert.com -td sha256 \
            -fd sha256 -d "$DESCRIPTION" \
            -du "https://github.com/mullvad/mullvadvpn-app#readme" \
            -sha1 "$CERT_HASH" "$binary"
        then
            break
        fi

        if [ "$i" -eq "${NUM_RETRIES}" ]; then
            log_error "Failed to sign $binary"
            exit 1
        fi

        sleep 1
    done
done
