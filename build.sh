#!/usr/bin/env bash

# This script is used to build, and sign a release artifact. See `README.md` for further
# instructions.
#
# Invoke the script with --dev-build in order to skip checks, cleaning and signing.

set -eu

function log {
    local NO_COLOR="0m"
    local msg=$1
    local color=${2:-"$NO_COLOR"}
    echo -e "\033[$color$msg\033[$NO_COLOR"
}

function log_header {
    local YELLOW="33m"
    echo ""
    log "### $1 ###" $YELLOW
    echo ""
}

function log_success {
    local GREEN="32m"
    log "$1" $GREEN
}

function log_error {
    local RED="31m"
    log "!! $1" $RED
}

function log_info {
    local BOLD="1m"
    log "$1" $BOLD
}

################################################################################
# Verify and configure environment.
################################################################################

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"
RUSTC_VERSION=$(rustc --version)
PRODUCT_VERSION=$(node -p "require('./gui/package.json').version" | sed -Ee 's/\.0//g')
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"target"}

CARGO_ARGS=()
NPM_PACK_ARGS=()

BUILD_MODE="release"
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --dev-build)
            BUILD_MODE="dev"
            ;;
        --universal)
            if [[ "$(uname -s)" == "Darwin" ]]; then
                TARGETS=(x86_64-apple-darwin aarch64-apple-darwin)
                NPM_PACK_ARGS+=(--universal)
            else
                log_error "--universal only works on macOS"
                exit 1
            fi
            ;;
        *)
            log_error "Unknown parameter: $1"
            exit 1
            ;;
    esac
    shift
done

if [[ "$BUILD_MODE" == "release" ]]; then
    if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
        log_error "Dirty working directory!"
        log_error "Will only build a signed app in a clean working directory"
        exit 1
    fi

    if [[ "$(uname -s)" == "Darwin" || "$(uname -s)" == "MINGW"* ]]; then
        log_info "Configuring environment for signing of binaries"
        if [[ -z ${CSC_LINK-} ]]; then
            log_error "The variable CSC_LINK is not set. It needs to point to a file containing the"
            log_error "private key used for signing of binaries."
            exit 1
        fi
        if [[ -z ${CSC_KEY_PASSWORD-} ]]; then
            read -spr "CSC_KEY_PASSWORD = " CSC_KEY_PASSWORD
            echo ""
            export CSC_KEY_PASSWORD
        fi
        # macOS: This needs to be set to 'true' to activate signing, even when CSC_LINK is set.
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
    NPM_PACK_ARGS+=(--no-compression)
    log_info "!! Development build. Not for general distribution !!"
    unset CSC_LINK CSC_KEY_PASSWORD
    export CSC_IDENTITY_AUTO_DISCOVERY=false
fi

product_version_commit_hash=$(git rev-parse "$PRODUCT_VERSION^{commit}" || echo "")
current_head_commit_hash=$(git rev-parse "HEAD^{commit}")
if [[ "$BUILD_MODE" == "dev" || $product_version_commit_hash != "$current_head_commit_hash" ]]; then
    PRODUCT_VERSION="$PRODUCT_VERSION-dev-${current_head_commit_hash:0:6}"

    log_info "Disabling Apple notarization (macOS only) of installer in this dev build"
    NPM_PACK_ARGS+=(--no-apple-notarization)
    CARGO_ARGS+=(--features api-override)
else
    log_info "Removing old Rust build artifacts..."
    cargo clean
    CARGO_ARGS+=(--locked)
fi

log_header "Building Mullvad VPN $PRODUCT_VERSION"

if [[ ("$(uname -s)" == "Darwin") ]]; then
    BINARIES=(
        mullvad-daemon
        mullvad
        mullvad-problem-report
        libtalpid_openvpn_plugin.dylib
        mullvad-setup
    )
elif [[ ("$(uname -s)" == "Linux") ]]; then
    BINARIES=(
        mullvad-daemon
        mullvad
        mullvad-problem-report
        libtalpid_openvpn_plugin.so
        mullvad-setup
        mullvad-exclude
    )
elif [[ ("$(uname -s)" == "MINGW"*) ]]; then
    BINARIES=(
        mullvad-daemon.exe
        mullvad.exe
        mullvad-problem-report.exe
        talpid_openvpn_plugin.dll
        mullvad-setup.exe
    )
fi

function restore_metadata_backups {
    pushd "$SCRIPT_DIR"
    log_info "Restoring version metadata files..."
    ./version-metadata.sh restore-backup --desktop
    mv Cargo.lock.bak Cargo.lock || true
    popd
}
trap 'restore_metadata_backups' EXIT

log_info "Updating version in metadata files..."
cp Cargo.lock Cargo.lock.bak
./version-metadata.sh inject "$PRODUCT_VERSION" --desktop

function sign_win {
    local NUM_RETRIES=3

    for binary in "$@"; do
        # Try multiple times in case the timestamp server cannot
        # be contacted.
        for i in $(seq 0 ${NUM_RETRIES}); do
            if signtool sign \
                -tr http://timestamp.digicert.com -td sha256 \
                -fd sha256 -d "Mullvad VPN" \
                -du "https://github.com/mullvad/mullvadvpn-app#readme" \
                -f "$CERT_FILE" \
                -p "$CERT_PASSPHRASE" "$binary"
            then
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

# Build the daemon and other Rust/C++ binaries, optionally
# sign them, strip them of debug symbols and copy to `dist-assets/`.
function build {
    local current_target=${1:-""}
    local for_target_string=""
    if [[ -n $current_target ]]; then
        for_target_string=" for $current_target"
    fi

    ################################################################################
    # Compile and link all binaries.
    ################################################################################

    if [[ "$(uname -s)" == "MINGW"* ]]; then
        CPP_BUILD_MODES="Release" ./build-windows-modules.sh "$@"
    fi

    ################################################################################
    # Compile wireguard-go
    ################################################################################

    ./wireguard/build-wireguard-go.sh "$current_target"

    export MULLVAD_ADD_MANIFEST="1"

    log_header "Building Rust code in release mode using $RUSTC_VERSION$for_target_string"

    CARGO_TARGET_ARG=()
    if [[ -n $current_target ]]; then
        CARGO_TARGET_ARG+=(--target="$current_target")
    fi

    cargo build "${CARGO_TARGET_ARG[@]}" "${CARGO_ARGS[@]}" --release

    ################################################################################
    # Move binaries to correct locations in dist-assets
    ################################################################################

    for binary in ${BINARIES[*]}; do
        if [[ -n $current_target ]]; then
            SRC="$CARGO_TARGET_DIR/$current_target/release/$binary"
        else
            SRC="$CARGO_TARGET_DIR/release/$binary"
        fi
        if [[ "$(uname -s)" == "Darwin" ]]; then
            # To make it easier to package universal builds on macOS the binaries are located in a
            # directory with the name of the target triple.
            DST_DIR="dist-assets/$current_target"
            DST="$DST_DIR/$binary"
            mkdir -p "$DST_DIR"
        else
            DST="dist-assets/$binary"
        fi

        if [[ "$BUILD_MODE" == "release" && "$(uname -s)" == "MINGW"* ]]; then
            sign_win "$SRC"
        fi

        if [[ "$(uname -s)" == "MINGW"* || "$binary" == *.dylib ]]; then
            log_info "Copying $SRC => $DST"
            cp "$SRC" "$DST"
        else
            log_info "Stripping $SRC => $DST"
            strip "$SRC" -o "$DST"
        fi
    done
}

if [[ "$(uname -s)" == "Darwin" || "$(uname -s)" == "Linux" ]]; then
    mkdir -p "dist-assets/shell-completions"
    for sh in bash zsh fish; do
        log_info "Generating shell completion script for $sh..."
        cargo run --bin mullvad "${CARGO_ARGS[@]}" --release -- shell-completions "$sh" \
            "dist-assets/shell-completions/"
    done
fi

./update-relays.sh
./update-api-address.sh

# Compile for all defined targets, or the current architecture if unspecified.
if [[ -n ${TARGETS:-""} ]]; then
    for t in ${TARGETS[*]}; do
        source env.sh "$t"
        build "$t"
    done
else
    source env.sh ""
    build
fi

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


log_header "Installing JavaScript dependencies"

pushd gui
npm ci

################################################################################
# Package release.
################################################################################

log_header "Packing final release artifact(s)"

case "$(uname -s)" in
    Linux*)     npm run pack:linux -- "${NPM_PACK_ARGS[@]}";;
    Darwin*)    npm run pack:mac -- "${NPM_PACK_ARGS[@]}";;
    MINGW*)     npm run pack:win -- "${NPM_PACK_ARGS[@]}";;
esac

popd

SEMVER_VERSION=$(echo "$PRODUCT_VERSION" | sed -Ee 's/($|-.*)/.0\1/g')
for semver_path in dist/*"$SEMVER_VERSION"*; do
    product_path=$(echo "$semver_path" | sed -Ee "s/$SEMVER_VERSION/$PRODUCT_VERSION/g")
    log_info "Moving $semver_path -> $product_path"
    mv "$semver_path" "$product_path"

    if [[ "$BUILD_MODE" == "release" && "$(uname -s)" == "MINGW"* && "$product_path" == *.exe ]]
    then
        # sign installer
        sign_win "$product_path"
    fi
done


log_success "**********************************"
log_success ""
log_success " The build finished successfully! "
log_success " You have built:"
log_success ""
log_success " $PRODUCT_VERSION"
log_success ""
log_success "**********************************"
