# shellcheck shell=bash
#
# Sourcing this file should set up the environment to build the app

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

source "$SCRIPT_DIR/scripts/utils/host"

ENV_TARGET=${1:-$HOST}

case "$ENV_TARGET" in
  x86_64-*-darwin*)
    export MACOSX_DEPLOYMENT_TARGET="10.12"

    if [[ $HOST != "$ENV_TARGET" ]]; then
        # Required for building daemon
        SDKROOT=$(xcrun --show-sdk-path)
        export SDKROOT
    fi
    ;;
  aarch64-*-darwin*)
    export MACOSX_DEPLOYMENT_TARGET="11.0"

    if [[ $HOST != "$ENV_TARGET" ]]; then
        # Required for building daemon
        SDKROOT=$(xcrun --show-sdk-path)
        export SDKROOT
    fi
    ;;
  *linux*)
    ;;
  *windows*)
    ;;
  *)
    echo "Unknown target \"$ENV_TARGET\"" >&2
    exit 1
    ;;
esac
