#!/usr/bin/env bash

# This script is used to build, and sign a release artifact. See `README.md` for instructions on
# how to just build a development/testing version.
#
# Invoke the script with --dev-build in order to skip checks, cleaning and signing.

################################################################################
# Platform specific configuration.
################################################################################

case "$(uname -s)" in
    Linux*)
        # config
        ;;
    Darwin*)
        export MACOSX_DEPLOYMENT_TARGET="10.7"
        ;;
    MINGW*)
        # config
        ;;
esac

################################################################################
# Verify and configure environment.
################################################################################

RUSTC_VERSION=`rustc +stable --version`

if [[ "${1:-""}" != "--dev-build" ]]; then

    REQUIRED_RUSTC_VERSION="rustc 1.26.2 (594fb253c 2018-06-01)"

    if [[ $RUSTC_VERSION != $REQUIRED_RUSTC_VERSION ]]; then
        echo "You are running the wrong Rust compiler version."
        echo "You are running $RUSTC_VERSION, but this project requires $REQUIRED_RUSTC_VERSION"
        echo "for release builds."
        exit 1
    fi

    if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
        echo "Dirty working directory!"
        echo "You should only build releases in clean working directories in order to make it"
        echo "easier to reproduce the same build."
        exit 1
    fi

    if [[ ("$(uname -s)" == "Darwin") || ("$(uname -s)" == "MINGW"*) ]]; then
        echo "Configuring environment for signing of binaries"
        if [[ -z ${CSC_LINK-} ]]; then
            echo "The variable CSC_LINK is not set. It needs to point to a file containing the"
            echo "private key used for signing of binaries."
            exit 1
        fi
        if [[ -z ${CSC_KEY_PASSWORD-} ]]; then
            read -sp "CSC_KEY_PASSWORD = " CSC_KEY_PASSWORD
            echo ""
            export CSC_KEY_PASSWORD
        fi
        # MacOs: This needs to be set to 'true' to activate signing, even when CSC_LINK is set.
        export CSC_IDENTITY_AUTO_DISCOVERY=true
    else
        unset CSC_LINK CSC_KEY_PASSWORD
        export CSC_IDENTITY_AUTO_DISCOVERY=false
    fi

    cargo +stable clean

else
    echo "!! Development build. Not for general distribution !!"
    unset CSC_LINK CSC_KEY_PASSWORD
    export CSC_IDENTITY_AUTO_DISCOVERY=false
fi

################################################################################
# Compile and link all binaries.
################################################################################

if [[ "$(uname -s)" == "MINGW"* ]]; then
    ./build_windows_libraries.sh $1
fi

echo "Building Rust code in release mode using $RUSTC_VERSION..."
cargo +stable build --release

################################################################################
# Other work to prepare the release.
################################################################################

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

################################################################################
# Package release.
################################################################################

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
