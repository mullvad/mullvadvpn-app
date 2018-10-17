# Sourcing this file should set up the environment to build the app

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

case "$(uname -s)" in
  Linux*)
    export LIBMNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/linux"
    export LIBNFTNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/linux"
    PLATFORM="linux"
    ;;
  Darwin*)
    export MACOSX_DEPLOYMENT_TARGET="10.7"
    PLATFORM="macos"
    ;;
  MINGW*|MSYS_NT*)
    PLATFORM="windows"
    ;;
esac

export OPENSSL_STATIC="1"
export OPENSSL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/$PLATFORM"
export OPENSSL_INCLUDE_DIR="$SCRIPT_DIR/dist-assets/binaries/$PLATFORM/include"
