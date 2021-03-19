# shellcheck shell=bash
#
# Sourcing this file should set up the environment to build the app

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

case "$(uname -s)" in
  Linux*)
    TARGET="x86_64-unknown-linux-gnu"
    ;;
  Darwin*)
    TARGET="x86_64-apple-darwin"
    ;;
  MINGW*|MSYS_NT*)
    TARGET="x86_64-pc-windows-msvc"
    ;;
esac

case "$TARGET" in
  *linux*)
    export LIBMNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/$TARGET"
    export LIBNFTNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/$TARGET"
    ;;
  *darwin*)
    export MACOSX_DEPLOYMENT_TARGET="10.7"
    ;;
  *windows*)
    ;;
  *)
    echo "Unknown target \"$TARGET\"" >&2
    exit 1
    ;;
esac
