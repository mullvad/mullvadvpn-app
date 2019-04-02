# Sourcing this file should set up the environment to build the app

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

if [ -n "$1" ]; then
    PLATFORM="$1"
fi

if [ -z "$PLATFORM" ]; then
    case "$(uname -s)" in
      Linux*)
        PLATFORM="linux"
        ;;
      Darwin*)
        PLATFORM="macos"
        ;;
      MINGW*|MSYS_NT*)
        PLATFORM="windows"
        ;;
    esac
fi

case "$PLATFORM" in
  linux)
    export LIBMNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/linux"
    export LIBNFTNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/linux"
    ;;
  macos)
    export MACOSX_DEPLOYMENT_TARGET="10.7"
    ;;
  windows)
    ;;
  android*)
    ;;
  *)
    echo "Unknown target platform \"$PLATFORM\"" >&2
    exit 1
    ;;
esac

export OPENSSL_STATIC="1"
export OPENSSL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/$PLATFORM"
export OPENSSL_INCLUDE_DIR="$SCRIPT_DIR/dist-assets/binaries/$PLATFORM/include"
