# Sourcing this file should set up the environment to build the app

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

export RUST_BUILD_PACKAGES=(
  mullvad-daemon
  mullvad-cli
  mullvad-setup
  mullvad-problem-report
  talpid-openvpn-plugin
)

function rust_package_args {
    for pkg in $@; do printf "-p $pkg "; done
}

function rust_build_package_args {
    rust_package_args $RUST_PACKAGES
}

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
    export RUST_TEST_EXCLUDE_PACKAGES=(mullvad-jni)
    ;;
  *darwin*)
    export MACOSX_DEPLOYMENT_TARGET="10.7"
    export RUST_TEST_EXCLUDE_PACKAGES=(talpid-dbus,mullvad-jni)
    ;;
  *windows*)
    export RUST_TEST_EXCLUDE_PACKAGES=(talpid-dbus,mullvad-jni)
    ;;
  *)
    echo "Unknown target \"$TARGET\"" >&2
    exit 1
    ;;
esac
