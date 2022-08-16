# shellcheck shell=bash
#
# Sourcing this file should set up the environment to build the app

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

case "$(uname -s)" in
  Linux*)
    arch="$(uname -m)"
    HOST="${arch}-unknown-linux-gnu"
    ;;
  Darwin*)
    arch="$(uname -m)"
    if [[ ("${arch}" == "arm64") ]]; then
        arch="aarch64"
    fi
    HOST="${arch}-apple-darwin"
    ;;
  MINGW*|MSYS_NT*)
    HOST="x86_64-pc-windows-msvc"
    ;;
esac

ENV_TARGET=${1:-$HOST}

case "$ENV_TARGET" in
  *linux*)
    export LIBMNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/$ENV_TARGET"
    export LIBNFTNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/$ENV_TARGET"
    ;;
  x86_64-*-darwin*)
    export MACOSX_DEPLOYMENT_TARGET="10.7"
    ;;
  aarch64-*-darwin*)
    export MACOSX_DEPLOYMENT_TARGET="11.0"

    if [[ $HOST != "$ENV_TARGET" ]]; then
        # Required for building daemon
        SDKROOT=$(xcrun --show-sdk-path)
        export SDKROOT
    fi
    ;;
  *windows*)
    ;;
  *)
    echo "Unknown target \"$ENV_TARGET\"" >&2
    exit 1
    ;;
esac
