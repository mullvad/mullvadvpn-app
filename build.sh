#!/usr/bin/env bash

# This script is used to build, and optionally sign the app.
# See `README.md` for further instructions.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

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
# Analyze environment and parse arguments
################################################################################

RUSTC_VERSION=$(rustc --version)
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"target"}

PRODUCT_VERSION=$(cd gui/; node -p "require('./package.json').version" | sed -Ee 's/\.0//g')

# If compiler optimization and artifact compression should be turned on or not
OPTIMIZE="false"
# If the produced binaries should be signed (Windows + macOS only)
SIGN="false"
# If a macOS build should create an installer artifact working on both
# Intel and Apple Silicon Macs
UNIVERSAL="false"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --optimize) OPTIMIZE="true";;
        --sign)     SIGN="true";;
        --universal)
            if [[ "$(uname -s)" != "Darwin" ]]; then
                log_error "--universal only works on macOS"
                exit 1
            fi
            UNIVERSAL="true"
            ;;
        *)
            log_error "Unknown parameter: $1"
            exit 1
            ;;
    esac
    shift
done

# Check if we are a building a release. Meaning we are configured to build with optimizations,
# sign the artifacts, AND we are currently building on a release git tag.
# Everything that is not a release build is called a "dev build" and has "-dev-{commit hash}"
# appended to the version name.
IS_RELEASE="false"
product_version_commit_hash=$(git rev-parse "$PRODUCT_VERSION^{commit}" || echo "")
current_head_commit_hash=$(git rev-parse "HEAD^{commit}")
if [[ "$SIGN" == "true" && "$OPTIMIZE" == "true" && \
      $product_version_commit_hash == "$current_head_commit_hash" ]]; then
    IS_RELEASE="true"
fi

################################################################################
# Configure build
################################################################################

CARGO_ARGS=()
NPM_PACK_ARGS=()

if [[ "$UNIVERSAL" == "true" ]]; then
    TARGETS=(x86_64-apple-darwin aarch64-apple-darwin)
    NPM_PACK_ARGS+=(--universal)
fi

if [[ "$OPTIMIZE" == "true" ]]; then
    CARGO_ARGS+=(--release)
    RUST_BUILD_MODE="release"
    CPP_BUILD_MODE="Release"
    NPM_PACK_ARGS+=(--release)
else
    RUST_BUILD_MODE="debug"
    NPM_PACK_ARGS+=(--no-compression)
    CPP_BUILD_MODE="Debug"
fi

if [[ "$SIGN" == "true" ]]; then
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
    log_info "!! Unsigned build. Not for general distribution !!"
    unset CSC_LINK CSC_KEY_PASSWORD
    export CSC_IDENTITY_AUTO_DISCOVERY=false
fi

if [[ "$IS_RELEASE" == "true" ]]; then
    log_info "Removing old Rust build artifacts..."
    cargo clean

    # Will not allow an outdated lockfile in releases
    CARGO_ARGS+=(--locked)
else
    PRODUCT_VERSION="$PRODUCT_VERSION-dev-${current_head_commit_hash:0:6}"

    # Allow dev builds to override which API server to use at runtime.
    CARGO_ARGS+=(--features api-override)

    if [[ "$(uname -s)" == "Darwin" ]]; then
        log_info "Disabling Apple notarization of installer in dev build"
        NPM_PACK_ARGS+=(--no-apple-notarization)
    fi
fi

# Make Windows builds include a manifest in the daemon binary declaring it must
# be run as admin.
if [[ "$(uname -s)" == "MINGW"* ]]; then
    export MULLVAD_ADD_MANIFEST="1"
fi

################################################################################
# Compile and build
################################################################################

log_header "Building Mullvad VPN $PRODUCT_VERSION"

function restore_metadata_backups {
    pushd "$SCRIPT_DIR" > /dev/null
    log_info "Restoring version metadata files..."
    ./version-metadata.sh restore-backup --desktop
    mv Cargo.lock.bak Cargo.lock || true
    popd > /dev/null
}
trap 'restore_metadata_backups' EXIT

log_info "Updating version in metadata files..."
cp Cargo.lock Cargo.lock.bak
./version-metadata.sh inject "$PRODUCT_VERSION" --desktop


# Sign all binaries passed as arguments to this function
function sign_win {
    local NUM_RETRIES=3

    for binary in "$@"; do
        # Try multiple times in case the timestamp server cannot
        # be contacted.
        for i in $(seq 0 ${NUM_RETRIES}); do
            log_info "Signing $binary..."
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

    log_header "Building wireguard-go$for_target_string"

    ./wireguard/build-wireguard-go.sh "$current_target"
    if [[ "$SIGN" == "true" && "$(uname -s)" == "MINGW"* ]]; then
        # Windows can only be built for this one target anyway, so it can be hardcoded.
        sign_win "build/lib/x86_64-pc-windows-msvc/libwg.dll"
    fi

    log_header "Building Rust code in $RUST_BUILD_MODE mode using $RUSTC_VERSION$for_target_string"

    local cargo_target_arg=()
    if [[ -n $current_target ]]; then
        cargo_target_arg+=(--target="$current_target")
    fi
    cargo build "${cargo_target_arg[@]}" "${CARGO_ARGS[@]}"

    ################################################################################
    # Move binaries to correct locations in dist-assets
    ################################################################################

    # All the binaries produced by cargo that we want to include in the app
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

    if [[ -n $current_target ]]; then
        local cargo_output_dir="$CARGO_TARGET_DIR/$current_target/$RUST_BUILD_MODE"
        # To make it easier to package universal builds on macOS the binaries are located in a
        # directory with the name of the target triple.
        local destination_dir="dist-assets/$current_target"
        mkdir -p "$destination_dir"
    else
        local cargo_output_dir="$CARGO_TARGET_DIR/$RUST_BUILD_MODE"
        local destination_dir="dist-assets"
    fi

    for binary in ${BINARIES[*]}; do
        local source="$cargo_output_dir/$binary"
        local destination="$destination_dir/$binary"

        if [[ "$(uname -s)" == "MINGW"* || "$binary" == *.dylib ]]; then
            log_info "Copying $source => $destination"
            cp "$source" "$destination"
        else
            log_info "Stripping $source => $destination"
            strip "$source" -o "$destination"
        fi

        if [[ "$SIGN" == "true" && "$(uname -s)" == "MINGW"* ]]; then
            sign_win "$destination"
        fi
    done
}

if [[ "$(uname -s)" == "MINGW"* ]]; then
    log_header "Building C++ code in $CPP_BUILD_MODE mode"
    CPP_BUILD_MODES=$CPP_BUILD_MODE IS_RELEASE=$IS_RELEASE ./build-windows-modules.sh

    if [[ "$SIGN" == "true" ]]; then
        CPP_BINARIES=(
            "windows/winfw/bin/x64-$CPP_BUILD_MODE/winfw.dll"
            "windows/windns/bin/x64-$CPP_BUILD_MODE/windns.dll"
            "windows/winnet/bin/x64-$CPP_BUILD_MODE/winnet.dll"
            "windows/driverlogic/bin/x64-$CPP_BUILD_MODE/driverlogic.exe"
            # The nsis plugin is always built in 32 bit release mode
            windows/nsis-plugins/bin/Win32-Release/*.dll
        )
        sign_win "${CPP_BINARIES[@]}"
    fi
fi

# Compile for all defined targets, or the current architecture if unspecified.
if [[ -n ${TARGETS:-""} ]]; then
    for t in ${TARGETS[*]}; do
        source env.sh "$t"
        build "$t"
    done
else
    source env.sh ""
    if [[ "$(uname -s)" == "Darwin" ]]; then
        # Provide target for non-universal macOS builds to use the same output location as for
        # universal builds
        build "$ENV_TARGET"
    else
        build
    fi
fi

################################################################################
# Package app.
################################################################################

log_header "Preparing for packaging Mullvad VPN $PRODUCT_VERSION"

if [[ "$(uname -s)" == "Darwin" || "$(uname -s)" == "Linux" ]]; then
    mkdir -p "dist-assets/shell-completions"
    for sh in bash zsh fish; do
        log_info "Generating shell completion script for $sh..."
        cargo run --bin mullvad "${CARGO_ARGS[@]}" -- shell-completions "$sh" \
            "dist-assets/shell-completions/"
    done
fi

log_info "Updating relays.json..."
cargo run --bin relay_list "${CARGO_ARGS[@]}" > dist-assets/relays.json


log_header "Installing JavaScript dependencies"

pushd gui
npm ci

log_header "Packing Mullvad VPN $PRODUCT_VERSION artifact(s)"

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

    if [[ "$SIGN" == "true" && "$(uname -s)" == "MINGW"* && "$product_path" == *.exe ]]; then
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
