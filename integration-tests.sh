#!/usr/bin/env bash

case "$(uname -s)" in
    Linux*)
        if [ "$UID" -ne 0 ]; then
            echo "WARNING: Not running as root, some tests may fail" >&2
        fi
        ;;
    MINGW*)
        if ! net session &> /dev/null; then
            echo "WARNING: Not running as administrator, some tests may fail" >&2
        fi
        ;;
    *)
        echo "ERROR: Platform $OSTYPE not supported"
        exit 1
        ;;
esac

MULLVAD_DIR="$(cd "$(dirname "$0")"; pwd -P)"

pushd "$MULLVAD_DIR"

cargo build \
    && cd mullvad-tests \
    && cargo test --features "integration-tests" -- --test-threads=1

RESULT="$?"
popd
exit "$RESULT"
