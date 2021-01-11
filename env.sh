# Sourcing this file should set up the environment to build the app

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"


function rust_package_args {
    for pkg in "$@"; do echo "-p ${pkg} "; done
}

function cargo_build_crate_args {
    local RUST_BUILD_CRATES=(
      mullvad-daemon
      mullvad-cli
      mullvad-setup
      mullvad-problem-report
      talpid-openvpn-plugin
    )
    rust_package_args "${RUST_BUILD_CRATES[@]}"
}

function cargo_test_crate_args {
    local ALL_RUST_PACKAGES=$( cd $SCRIPT_DIR;
        (for manifest in $(find */ -name Cargo.toml | grep -v dist-assets);
            do basename $(dirname $manifest); done) )

    rust_package_args $((echo ${ALL_RUST_PACKAGES[@]}; echo ${RUST_TEST_EXCLUDE_PACKAGES[@]}) | \
        tr " " "\n" | \
        sort | \
        uniq -u)
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
    export RUST_TEST_EXCLUDE_PACKAGES=(mullvad-jni mullvad-tests)
    ;;
  *darwin*)
    export MACOSX_DEPLOYMENT_TARGET="10.7"
    export RUST_TEST_EXCLUDE_PACKAGES=(talpid-dbus mullvad-jni mullvad-tests)
    ;;
  *windows*)
    export RUST_TEST_EXCLUDE_PACKAGES=(talpid-dbus mullvad-jni mullvad-tests)
    ;;
  *)
    echo "Unknown target \"$TARGET\"" >&2
    exit 1
    ;;
esac
