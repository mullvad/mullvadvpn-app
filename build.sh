#!/usr/bin/env bash

# This script is used to build, and optionally sign the app.
# See `README.md` for further instructions.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

source scripts/utils/host
source scripts/utils/log

################################################################################
# Analyze environment and parse arguments
################################################################################

RUSTC_VERSION=$(rustc --version)
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"target"}

echo "Computing build version..."
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version)
log_header "Building Mullvad VPN $PRODUCT_VERSION"

# If compiler optimization and artifact compression should be turned on or not
OPTIMIZE="false"
# If the produced binaries should be signed (Windows + macOS only)
SIGN="false"
# If the produced app and pkg should be notarized by apple (macOS only)
NOTARIZE="false"
# If a macOS or Windows build should create an installer artifact working on both
# x86 and arm64
UNIVERSAL="false"
# Use boringtun instead of wireguard-go
BORINGTUN="false"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --optimize) OPTIMIZE="true";;
        --sign)     SIGN="true";;
        --notarize) NOTARIZE="true";;
        --universal)
            if [[ "$(uname -s)" != "Darwin" && "$(uname -s)" != "MINGW"* ]]; then
                log_error "--universal only works on macOS and Windows"
                exit 1
            fi
            UNIVERSAL="true"
            ;;
        --boringtun) BORINGTUN="true";;
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
if [[ "$SIGN" == "true" && "$OPTIMIZE" == "true" && "$PRODUCT_VERSION" != *"-dev-"* ]]; then
    IS_RELEASE="true"
fi

################################################################################
# Configure build
################################################################################

CARGO_ARGS=()
NPM_PACK_ARGS=()

if [[ -n ${TARGETS:-""} ]]; then
    NPM_PACK_ARGS+=(--targets "${TARGETS[*]}")
fi

NPM_PACK_ARGS+=(--host-target-triple "$HOST")


if [[ "$UNIVERSAL" == "true" ]]; then
    if [[ -n ${TARGETS:-""} ]]; then
        log_error "'TARGETS' and '--universal' cannot be specified simultaneously."
        exit 1
    else
        log_info "Building universal distribution"
    fi

    # Universal builds package targets for both aarch64 and x86_64. We leave the target
    # corresponding to the host machine empty to avoid rebuilding multiple times.
    # When the --target flag is provided to cargo it always puts the build in the target/$ENV_TARGET
    # folder even when it matches you local machine, as opposed to just the target folder.
    # This causes the cached build not to get used when later running e.g.
    # 'cargo run --bin mullvad --shell-completions'.
    case $HOST in
        x86_64-apple-darwin) TARGETS=("" aarch64-apple-darwin);;
        aarch64-apple-darwin) TARGETS=("" x86_64-apple-darwin);;
        x86_64-pc-windows-msvc) TARGETS=("" aarch64-pc-windows-msvc);;
        aarch64-pc-windows-msvc) TARGETS=("" x86_64-pc-windows-msvc);;
    esac

    NPM_PACK_ARGS+=(--universal)
fi

if [[ "$OPTIMIZE" == "true" ]]; then
    CARGO_ARGS+=(--release)
    RUST_BUILD_MODE="release"
    NPM_PACK_ARGS+=(--release)
else
    RUST_BUILD_MODE="debug"
    NPM_PACK_ARGS+=(--no-compression)
fi
# The cargo builds that are part of the C++ builds only enforce `--locked` when built
# in release mode. And we must enforce `--locked` for all signed builds. So we enable
# release mode if either optimizations or signing is enabled.
if [[ "$OPTIMIZE" == "true" || "$SIGN" == "true" ]]; then
    CPP_BUILD_MODE="Release"
else
    CPP_BUILD_MODE="Debug"
fi

function assert_clean_working_directory {
    if [[ -n "$(git status --porcelain)" ]]; then
        log_error "Dirty working directory!"
        log_error "Release builds are not allowed on dirty working directories!"
        exit 1
    fi
}

if [[ "$SIGN" == "true" ]]; then
    # Refuse to build signed builds on dirty working directories. Prevents release builds
    # from being built from potentially modified code/assets.
    assert_clean_working_directory

    # Will not allow an outdated lockfile when building with signatures
    # (The build servers should never build without --locked for
    # reproducibility and supply chain security)
    CARGO_ARGS+=(--locked)

    if [[ "$(uname -s)" == "Darwin" ]]; then
        log_info "Configuring environment for signing of binaries"
        if [[ -z ${CSC_LINK-} ]]; then
            log_error "The variable CSC_LINK is not set. It needs to point to a file containing the"
            log_error "private key used for signing of binaries."
            exit 1
        fi
        if [[ -z ${CSC_KEY_PASSWORD-} ]]; then
            read -rsp "CSC_KEY_PASSWORD = " CSC_KEY_PASSWORD
            echo ""
            export CSC_KEY_PASSWORD
        fi
        # macOS: This needs to be set to 'true' to activate signing, even when CSC_LINK is set.
        export CSC_IDENTITY_AUTO_DISCOVERY=true
    elif [[ "$(uname -s)" == "MINGW"* ]]; then
        if [[ -z ${CERT_HASH-} ]]; then
            log_error "The variable CERT_HASH is not set. It needs to be set to the thumbprint of"
            log_error "the signing certificate."
            exit 1
        fi

        unset CSC_LINK CSC_KEY_PASSWORD
        export CSC_IDENTITY_AUTO_DISCOVERY=false
    else
        unset CSC_LINK CSC_KEY_PASSWORD
        export CSC_IDENTITY_AUTO_DISCOVERY=false
    fi
else
    log_info "!! Unsigned build. Not for general distribution !!"
    unset CSC_LINK CSC_KEY_PASSWORD
    export CSC_IDENTITY_AUTO_DISCOVERY=false
fi

if [[ "$NOTARIZE" == "true" ]]; then
    NPM_PACK_ARGS+=(--notarize)
fi

if [[ "$IS_RELEASE" == "true" ]]; then
    log_info "Removing old Rust build artifacts..."
    cargo clean
else
    # Allow dev builds to override which API server to use at runtime.
    CARGO_ARGS+=(--features api-override)
fi

# Make Windows builds include a manifest in the daemon binary declaring it must
# be run as admin.
if [[ "$(uname -s)" == "MINGW"* ]]; then
    export MULLVAD_ADD_MANIFEST="1"
fi

################################################################################
# Compile and build
################################################################################

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
                -sha1 "$CERT_HASH" "$binary"
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
# sign them, and copy to `dist-assets/`.
function build {
    local specified_target=${1:-""}
    local current_target=${specified_target:-"$HOST"}
    local for_target_string
    if [[ -n $specified_target ]]; then
        for_target_string=" for $current_target"
    else
        for_target_string=" for local target $HOST"
    fi

    ################################################################################
    # Compile and link all binaries.
    ################################################################################

    log_header "Building Rust code in $RUST_BUILD_MODE mode using $RUSTC_VERSION$for_target_string"

    local cargo_target_arg=()
    if [[ -n $specified_target ]]; then
        cargo_target_arg+=(--target="$specified_target")
    fi

    local cargo_features=()
    if [[ "$BORINGTUN" == "false" ]]; then
        cargo_features+=(--features wireguard-go)
    fi

    local cargo_crates_to_build=(
        -p mullvad-daemon --bin mullvad-daemon
        -p mullvad-cli --bin mullvad
        -p mullvad-setup --bin mullvad-setup
        -p mullvad-problem-report --bin mullvad-problem-report
        -p talpid-openvpn-plugin --lib
    )
    if [[ ("$(uname -s)" == "Linux") ]]; then
        cargo_crates_to_build+=(-p mullvad-exclude --bin mullvad-exclude)
    fi

    cargo build "${cargo_target_arg[@]}" "${cargo_features[@]}" "${CARGO_ARGS[@]}" "${cargo_crates_to_build[@]}"

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
        if [[ "$BORINGTUN" == "false" ]]; then
            BINARIES+=(
                libwg.dll
                maybenot_ffi.dll
            )
        fi
    fi

    if [[ -n $specified_target ]]; then
        local cargo_output_dir="$CARGO_TARGET_DIR/$specified_target/$RUST_BUILD_MODE"
        # To make it easier to package multiple targets, the binaries are
        # located in a directory with the name of the target triple.
        local destination_dir="dist-assets/$specified_target"
        mkdir -p "$destination_dir"
    else
        local cargo_output_dir="$CARGO_TARGET_DIR/$RUST_BUILD_MODE"
        local destination_dir="dist-assets"
    fi

    for binary in "${BINARIES[@]}"; do
        local source="$cargo_output_dir/$binary"
        local destination="$destination_dir/$binary"

        log_info "Copying $source => $destination"
        cp "$source" "$destination"

        if [[ "$SIGN" == "true" && "$(uname -s)" == "MINGW"* ]]; then
            sign_win "$destination"
        fi
    done

    if [[ "$current_target" == "aarch64-pc-windows-msvc" ]]; then
        # We ship x64 OpenVPN with ARM64, so we need an x64 talpid-openvpn-plugin
        # to include in the package.
        local source="$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/$RUST_BUILD_MODE/talpid_openvpn_plugin.dll"
        local destination
        if [[ -n "$specified_target" ]]; then
            destination="dist-assets/$specified_target/talpid_openvpn_plugin.dll"
        else
            destination="dist-assets/talpid_openvpn_plugin.dll"
        fi

        log_info "Workaround: building x64 talpid-openvpn-plugin"
        cargo build --target x86_64-pc-windows-msvc "${CARGO_ARGS[@]}" -p talpid-openvpn-plugin --lib
        cp "$source" "$destination"
        if [[ "$SIGN" == "true" ]]; then
            sign_win "$destination"
        fi
    fi
}

if [[ "$(uname -s)" == "MINGW"* ]]; then
    if [[ "$IS_RELEASE" == "true" ]]; then
        ./build-windows-modules.sh clean
    else
        echo "Will NOT clean intermediate files in ./windows/**/bin/ in dev builds"
    fi

    for t in "${TARGETS[@]:-"$HOST"}"; do
        case "${t:-"$HOST"}" in
            x86_64-pc-windows-msvc) CPP_BUILD_TARGET=x64;;
            aarch64-pc-windows-msvc) CPP_BUILD_TARGET=ARM64;;
            *)
                log_error "Unknown Windows target: $t"
                exit 1
                ;;
        esac

        log_header "Building C++ code in $CPP_BUILD_MODE mode for $CPP_BUILD_TARGET"
        CPP_BUILD_MODES=$CPP_BUILD_MODE CPP_BUILD_TARGETS=$CPP_BUILD_TARGET ./build-windows-modules.sh

        if [[ "$SIGN" == "true" ]]; then
            CPP_BINARIES=(
                "windows/winfw/bin/$CPP_BUILD_TARGET-$CPP_BUILD_MODE/winfw.dll"
                "windows/driverlogic/bin/$CPP_BUILD_TARGET-$CPP_BUILD_MODE/driverlogic.exe"
                # The nsis plugin is always built in 32 bit release mode
                windows/nsis-plugins/bin/Win32-Release/*.dll
            )
            sign_win "${CPP_BINARIES[@]}"
        fi
    done
fi

for t in "${TARGETS[@]:-""}"; do
    source env.sh "$t"
    build "$t"
done


################################################################################
# Package app.
################################################################################

log_header "Preparing for packaging Mullvad VPN $PRODUCT_VERSION"

if [[ "$(uname -s)" == "Darwin" || "$(uname -s)" == "Linux" ]]; then
    mkdir -p "build/shell-completions"
    for sh in bash zsh fish; do
        log_info "Generating shell completion script for $sh..."
        cargo run --bin mullvad "${CARGO_ARGS[@]}" -- shell-completions "$sh" \
            "build/shell-completions/"
    done
else
    mkdir -p "build"
fi

log_info "Updating relays.json..."
cargo run -p mullvad-api --bin relay_list "${CARGO_ARGS[@]}" > build/relays.json


log_header "Installing JavaScript dependencies"

pushd desktop
npm ci --no-audit --no-fund

pushd packages/mullvad-vpn

log_header "Packing Mullvad VPN $PRODUCT_VERSION artifact(s)"

case "$(uname -s)" in
    Linux*)     npm run pack:linux -- "${NPM_PACK_ARGS[@]}";;
    Darwin*)    npm run pack:mac -- "${NPM_PACK_ARGS[@]}";;
    MINGW*)     npm run pack:win -- "${NPM_PACK_ARGS[@]}";;
esac
popd
popd

# When signing is enabled, we check that the working directory is clean before building,
# further up. Now verify that this is still true. The build process should never make the
# working directory dirty.
# This could for example happen if lockfiles are outdated, and the build process updates them.
if [[ "$SIGN" == "true" ]]; then
    assert_clean_working_directory
fi

# sign installer on Windows
if [[ "$SIGN" == "true" && "$(uname -s)" == "MINGW"* ]]; then
    for installer_path in dist/*"$PRODUCT_VERSION"*.exe; do
        log_info "Signing $installer_path"
        sign_win "$installer_path"
    done
fi

# pack universal installer on Windows
if [[ "$UNIVERSAL" == "true" && "$(uname -s)" == "MINGW"* ]]; then
    WIN_PACK_ARGS=()
    if [[ "$OPTIMIZE" == "true" ]]; then
        WIN_PACK_ARGS+=(--optimize)
    fi
    ./desktop/scripts/pack-universal-win.sh \
        --x64-installer "$SCRIPT_DIR/dist/"*"$PRODUCT_VERSION"_x64.exe \
        --arm64-installer "$SCRIPT_DIR/dist/"*"$PRODUCT_VERSION"_arm64.exe \
        "${WIN_PACK_ARGS[@]}"
    if [[ "$SIGN" == "true" ]]; then
        assert_clean_working_directory
        sign_win "dist/MullvadVPN-${PRODUCT_VERSION}.exe"
    fi
fi

# notarize installer on macOS
if [[ "$NOTARIZE" == "true" && "$(uname -s)" == "Darwin" ]]; then
    log_info "Notarizing pkg"
    xcrun notarytool submit dist/*"$PRODUCT_VERSION"*.pkg \
        --keychain "$NOTARIZE_KEYCHAIN" \
        --keychain-profile "$NOTARIZE_KEYCHAIN_PROFILE" \
        --wait

    log_info "Stapling pkg"
    xcrun stapler staple dist/*"$PRODUCT_VERSION"*.pkg
fi

log_success "**********************************"
log_success ""
log_success " The build finished successfully! "
log_success " You have built:"
log_success ""
log_success " $PRODUCT_VERSION"
log_success ""
log_success "**********************************"
