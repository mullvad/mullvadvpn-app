#!/usr/bin/env bash

# This script is used to build, and sign a release artifact. See `README.md` for further
# instructions.
#
# Invoke the script with --dev-build in order to skip checks, cleaning and signing.

set -eu

################################################################################
# Verify and configure environment.
################################################################################

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"
RUSTC_VERSION=`rustc +stable --version`
PRODUCT_VERSION=$(node -p "require('./gui/package.json').version" | sed -Ee 's/\.0//g')
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$SCRIPT_DIR/target"}

source env.sh

if [[ "${1:-""}" != "--dev-build" ]]; then
    BUILD_MODE="release"
    NPM_PACK_ARGS=""
    if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
        echo "Dirty working directory!"
        echo "You should only build releases in clean working directories in order to make it"
        echo "easier to reproduce the same build."
        exit 1
    fi

    if [[ ("$(uname -s)" == "Darwin") || "$(uname -s)" == "MINGW"* ]]; then
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

        if [[ "$(uname -s)" == "MINGW"* ]]; then
            CERT_FILE=$CSC_LINK
            CERT_PASSPHRASE=$CSC_KEY_PASSWORD
            unset CSC_LINK CSC_KEY_PASSWORD
            export CSC_IDENTITY_AUTO_DISCOVERY=false
        fi
    else
        unset CSC_LINK CSC_KEY_PASSWORD
        export CSC_IDENTITY_AUTO_DISCOVERY=false
    fi
else
    BUILD_MODE="dev"
    NPM_PACK_ARGS="--no-compression"
    echo "!! Development build. Not for general distribution !!"
    unset CSC_LINK CSC_KEY_PASSWORD
    export CSC_IDENTITY_AUTO_DISCOVERY=false
fi

if [[ "$BUILD_MODE" == "dev" || $(git describe) != "$PRODUCT_VERSION" ]]; then
    GIT_COMMIT=$(git rev-parse HEAD | head -c 6)
    PRODUCT_VERSION="$PRODUCT_VERSION-dev-$GIT_COMMIT"
    echo "Modifying product version to $PRODUCT_VERSION"

    echo "Disabling Apple notarization (macOs only) of installer in this dev build"
    NPM_PACK_ARGS+=" --no-apple-notarization"
    CARGO_ARGS=""
else
    echo "Removing old Rust build artifacts"
    cargo +stable clean
    CARGO_ARGS="--locked"
fi

sign_win() {
    NUM_RETRIES=3

    for binary in "$@"; do
        # Try multiple times in case the timestamp server cannot
        # be contacted.
        for i in $(seq 0 ${NUM_RETRIES}); do
            signtool sign \
            -tr http://timestamp.digicert.com -td sha256 \
            -fd sha256 -d "Mullvad VPN" \
            -du "https://github.com/mullvad/mullvadvpn-app#readme" \
            -f "$CERT_FILE" \
            -p "$CERT_PASSPHRASE" "$binary"

            if [ "$?" -eq "0" ]; then
                break
            fi

            if [ "$i" -eq "${NUM_RETRIES}" ]; then
                return 1
            fi

            sleep 1
        done
    done
    return 0
}

echo "Building Mullvad VPN $PRODUCT_VERSION"

function restore_metadata_backups() {
    pushd "$SCRIPT_DIR"
    echo "Restoring version metadata files..."
    ./version-metadata.sh restore-backup
    mv Cargo.lock.bak Cargo.lock || true
    popd
}
trap 'restore_metadata_backups' EXIT

echo "Updating version in metadata files..."
cp Cargo.lock Cargo.lock.bak
./version-metadata.sh inject $PRODUCT_VERSION


################################################################################
# Compile and link all binaries.
################################################################################

if [[ "$(uname -s)" == "MINGW"* ]]; then
    CPP_BUILD_MODES="Release" ./build_windows_modules.sh $@
fi

################################################################################
# Compile wireguard-go
################################################################################
./wireguard/build-wireguard-go.sh

echo "Building Rust code in release mode using $RUSTC_VERSION..."

if [[ ("$(uname -s)" == "Darwin") || ("$(uname -s)" == "Linux") ]]; then
    pushd mullvad-cli
    mkdir -p "$SCRIPT_DIR/dist-assets/shell-completions"
    for sh in bash zsh fish; do
        echo "Generating shell completion script for $sh..."
        cargo +stable run $CARGO_ARGS --release --features shell-completions -- \
            shell-completions "$sh" "$SCRIPT_DIR/dist-assets/shell-completions/"
    done
    popd
fi

MULLVAD_ADD_MANIFEST="1" cargo +stable build $CARGO_ARGS --release

################################################################################
# Other work to prepare the release.
################################################################################

if [[ ("$(uname -s)" == "Darwin") ]]; then
    binaries=(
        mullvad-daemon
        mullvad
        mullvad-problem-report
        libtalpid_openvpn_plugin.dylib
        mullvad-setup
    )
elif [[ ("$(uname -s)" == "Linux") ]]; then
    binaries=(
        mullvad-daemon
        mullvad
        mullvad-problem-report
        libtalpid_openvpn_plugin.so
        mullvad-setup
        mullvad-exclude
    )
elif [[ ("$(uname -s)" == "MINGW"*) ]]; then
    binaries=(
        mullvad-daemon.exe
        mullvad.exe
        mullvad-problem-report.exe
        talpid_openvpn_plugin.dll
        mullvad-setup.exe
    )
fi
for binary in ${binaries[*]}; do
    SRC="$CARGO_TARGET_DIR/release/$binary"
    DST="$SCRIPT_DIR/dist-assets/$binary"

    if [[ "$BUILD_MODE" == "release" && "$(uname -s)" == "MINGW"* ]]; then
        sign_win "$SRC"
    fi

    if [[ "$(uname -s)" == "MINGW"* || "$binary" == *.dylib ]]; then
        echo "Copying $SRC => $DST"
        cp "$SRC" "$DST"
    else
        echo "Stripping $SRC => $DST"
        strip "$SRC" -o "$DST"
    fi
done

if [[ "$BUILD_MODE" == "release" && "$(uname -s)" == "MINGW"* ]]; then
    signdep=(
        windows/winfw/bin/x64-Release/winfw.dll
        windows/windns/bin/x64-Release/windns.dll
        windows/winnet/bin/x64-Release/winnet.dll
        windows/driverlogic/bin/x64-Release/driverlogic.exe
        windows/nsis-plugins/bin/Win32-Release/*.dll
        build/lib/x86_64-pc-windows-msvc/libwg.dll
    )
    sign_win "${signdep[@]}"
fi


./update-relays.sh


pushd "$SCRIPT_DIR/gui"

echo "Installing JavaScript dependencies..."

# Add `--no-optional` flag when running on non-macOS environments because `npm ci` attempts to
# install optional dependencies that aren't even available on other platforms.
NPM_CI_ARGS=""
if [ "$(uname -s)" != "Darwin" ]; then
    NPM_CI_ARGS+="--no-optional"
fi

npm ci $NPM_CI_ARGS

################################################################################
# Package release.
################################################################################

echo "Packing final release artifact..."

case "$(uname -s)" in
    Linux*)     npm run pack:linux -- $NPM_PACK_ARGS;;
    Darwin*)    npm run pack:mac -- $NPM_PACK_ARGS;;
    MINGW*)     npm run pack:win -- $NPM_PACK_ARGS;;
esac

popd

SEMVER_VERSION=$(echo $PRODUCT_VERSION | sed -Ee 's/($|-.*)/.0\1/g')
for semver_path in dist/*$SEMVER_VERSION*; do
    product_path=$(echo $semver_path | sed -Ee "s/$SEMVER_VERSION/$PRODUCT_VERSION/g")
    echo "Moving $semver_path -> $product_path"
    mv $semver_path $product_path

    if [[ "$BUILD_MODE" == "release" && "$(uname -s)" == "MINGW"* && "$product_path" == *.exe ]]
    then
        # sign installer
        sign_win "$product_path"
    fi
done

echo "**********************************"
echo ""
echo " The build finished successfully! "
echo " You have built:"
echo ""
echo " $PRODUCT_VERSION"
echo ""
echo "**********************************"
