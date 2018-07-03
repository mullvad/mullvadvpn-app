#!/usr/bin/env bash

# This script is used to build, and sign a release artifact. See `README.md` for further
# instructions.
#
# Invoke the script with --dev-build in order to skip checks, cleaning and signing.

set -eu

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

################################################################################
# Platform specific configuration.
################################################################################

case "$(uname -s)" in
    Linux*)
        # Use static builds of libmnl and libnftnl from the binaries submodule
        export LIBMNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/linux"
        export LIBNFTNL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/linux"
        PLATFORM="linux"
        ;;
    Darwin*)
        export MACOSX_DEPLOYMENT_TARGET="10.7"
        PLATFORM="macos"
        ;;
    MINGW*)
        PLATFORM="windows"
        ;;
esac

################################################################################
# Verify and configure environment.
################################################################################

RUSTC_VERSION=`rustc +stable --version`
PRODUCT_VERSION=$(node -p "require('./package.json').version" | sed -Ee 's/\.0//g')

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
    GIT_COMMIT=$(git rev-parse --short HEAD)
    PRODUCT_VERSION="$PRODUCT_VERSION-dev-$GIT_COMMIT"

    unset CSC_LINK CSC_KEY_PASSWORD
    export CSC_IDENTITY_AUTO_DISCOVERY=false
fi

echo "Building Mullvad VPN $PRODUCT_VERSION"
SEMVER_VERSION=$(echo $PRODUCT_VERSION | sed -Ee 's/($|-.*)/.0\1/g')

function restore_metadata_backups() {
    mv package.json.bak package.json || true
    mv Cargo.lock.bak Cargo.lock || true
    mv mullvad-daemon/Cargo.toml.bak mullvad-daemon/Cargo.toml || true
    mv mullvad-cli/Cargo.toml.bak mullvad-cli/Cargo.toml || true
    mv mullvad-problem-report/Cargo.toml.bak mullvad-problem-report/Cargo.toml || true
}
trap 'restore_metadata_backups' EXIT

sed -i.bak \
    -Ee "s/\"version\": \"[^\"]+\",/\"version\": \"$SEMVER_VERSION\",/g" \
    package.json

cp Cargo.lock Cargo.lock.bak
sed -i.bak \
    -Ee "s/^version = \"[^\"]+\"\$/version = \"$SEMVER_VERSION\"/g" \
    mullvad-daemon/Cargo.toml \
    mullvad-cli/Cargo.toml \
    mullvad-problem-report/Cargo.toml

################################################################################
# Compile and link all binaries.
################################################################################

export OPENSSL_STATIC="1"
export OPENSSL_LIB_DIR="$SCRIPT_DIR/dist-assets/binaries/$PLATFORM"
export OPENSSL_INCLUDE_DIR="$SCRIPT_DIR/dist-assets/binaries/$PLATFORM/include"

if [[ "$(uname -s)" == "MINGW"* ]]; then
    CPP_BUILD_MODES="Release" ./build_windows_libraries.sh $@
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
cp dist-assets/api_root_ca.pem target/release/
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

for semver_path in dist/*$SEMVER_VERSION*; do
    product_path=$(echo $semver_path | sed -Ee "s/$SEMVER_VERSION/$PRODUCT_VERSION/g")
    echo "Moving $semver_path -> $product_path"
    mv $semver_path $product_path
done

echo "**********************************"
echo ""
echo " The build finished successfully! "
echo " You have built:"
echo ""
echo " $PRODUCT_VERSION"
echo ""
echo "**********************************"
