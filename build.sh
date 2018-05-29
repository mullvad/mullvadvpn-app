#!/usr/bin/env bash

# This script is used to build, and sign a release artifact. See `README.md` for instructions on
# how to just build a development/testing version.

set -eu

REQUIRED_RUSTC_VERSION="rustc 1.26.0 (a77568041 2018-05-07)"
RUSTC_VERSION=`rustc +stable --version`
if [[ $RUSTC_VERSION != $REQUIRED_RUSTC_VERSION ]]; then
    echo "You are running the wrong Rust compiler version."
    echo "You are running $RUSTC_VERSION, but this project requires $REQUIRED_RUSTC_VERSION"
    echo "for release builds."
    exit 1
fi

if [[ "${1:-""}" != "--allow-dirty" ]]; then
    if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
        echo "Dirty working directory!"
        echo "You should only build releases in clean working directories in order to make it"
        echo "easier to reproduce the same build."
        echo ""
        echo "Use --allow-dirty to skip this check. Never do this for official releases."
        exit 1
    fi
fi

if [[ "$(uname -s)" = "Darwin" ]]; then
    export MACOSX_DEPLOYMENT_TARGET="10.7"

    # if CSC_LINK is set, then we do signing
    if [[ ! -z ${CSC_LINK-} ]]; then
        echo "Building with macOS signing activated. Using certificate at $CSC_LINK"
        if [[ -z ${CSC_KEY_PASSWORD-} ]]; then
            read -sp "CSC_KEY_PASSWORD = " CSC_KEY_PASSWORD
            echo ""
            export CSC_KEY_PASSWORD
        fi
        export CSC_IDENTITY_AUTO_DISCOVERY=true
    else
        echo "!! CSC_LINK not set. This build will not be signed !!"
        unset CSC_LINK CSC_KEY_PASSWORD
        export CSC_IDENTITY_AUTO_DISCOVERY=false
    fi
fi

# Remove binaries. To make sure it is rebuilt with the stable toolchain and the latest changes.
cargo +stable clean

if [[ "$(uname -s)" == "MINGW"* ]]; then
    ./build_winfw.sh
fi

echo "Compiling mullvad-daemon in release mode with $RUSTC_VERSION..."
cargo +stable build --release

# Only strip binaries on platforms other than Windows.
if [[ "$(uname -s)" != "MINGW"* ]]; then
    binaries=(
        ./target/release/mullvad-daemon
        ./target/release/mullvad
        ./target/release/problem-report
    )
    for binary in ${binaries[*]}; do
        echo "Stripping debugging symbols from $binary"
        strip $binary
    done
fi

echo "Updating relay list..."
./target/release/list-relays > dist-assets/relays.json

echo "Installing JavaScript dependencies..."
yarn install

echo "Packing final release artifact..."
case "$(uname -s)" in
    Linux*)     yarn pack:linux;;
    Darwin*)    yarn pack:mac;;
    MINGW*)     yarn pack:win;;
esac

RELEASE_VERSION=`./target/release/mullvad-daemon --version | cut -f2 -d' '`
echo "**********************************"
echo ""
echo " The build finished successfully! "
echo " You have built:"
echo ""
echo " $RELEASE_VERSION"
echo ""
echo "**********************************"
