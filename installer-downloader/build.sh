#!/usr/bin/env bash

# This script is used to build, and optionally sign, the downloader, always in release mode.

# This script performs the equivalent of the following profile:
#
# [profile.release]
# strip = true
# opt-level = 'z'
# codegen-units = 1
# lto = true
# panic = 'abort'
#
# We cannot set all of the above directly in Cargo.toml since some must be set for the entire
# workspace.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# shellcheck disable=SC1091
source ../scripts/utils/host
# shellcheck disable=SC1091
source ../scripts/utils/log

CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"../target"}
export CARGO_TARGET_DIR

# Temporary build directory
BUILD_DIR="$SCRIPT_DIR/build"
# Successfully built (and signed) artifacts
DIST_DIR="$SCRIPT_DIR/../dist"

BUNDLE_NAME="MullvadVPNInstaller"
BUNDLE_ID="net.mullvad.$BUNDLE_NAME"

FILENAME="Install Mullvad VPN"

# When --upload is passed, git verify-tag looks for a signed tag with the prefix below.
# The signed tag must be named $TAG_PREFIX/<version>.
TAG_PREFIX="desktop/installer-downloader/"

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

mkdir -p "$DIST_DIR"

# Whether to sign and notarized produced binaries
SIGN="false"

# Whether to upload signed binaries
UPLOAD="false"

# Temporary keychain to store the .p12 in.
# This is automatically created/replaced when signing on macOS.
SIGN_KEYCHAIN_PATH="$HOME/Library/Keychains/mv-metadata-keychain-db"

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --sign)
            SIGN="true"
            ;;
        --upload)
            UPLOAD="true"
            ;;
        *)
            log_error "Unknown parameter: $1"
            exit 1
            ;;
    esac
    shift
done

if [[ "$UPLOAD" == "true" && "$SIGN" != "true" ]]; then
    log_error "'--upload' requires '--sign' to be specified"
    exit 1
fi

# Check that we have the correct environment set for signing
function assert_can_sign {
    if [[ "$(uname -s)" == "Darwin" ]]; then
        if [[ -z ${CSC_LINK-} ]]; then
            log_error "The variable CSC_LINK is not set. It needs to point to a file containing the private key used for signing of binaries."
            exit 1
        fi
        if [[ -z ${CSC_KEY_PASSWORD-} ]]; then
            read -rsp "CSC_KEY_PASSWORD = " CSC_KEY_PASSWORD
            echo ""
            export CSC_KEY_PASSWORD
        fi
        if [[ -z ${NOTARIZE_KEYCHAIN-} || -z ${NOTARIZE_KEYCHAIN_PROFILE-} ]]; then
            log_error "The variables NOTARIZE_KEYCHAIN and NOTARIZE_KEYCHAIN_PROFILE must be set."
            exit 1
        fi
    elif [[ "$(uname -s)" == "MINGW"* ]]; then
        if [[ -z ${CERT_HASH-} ]]; then
            log_error "The variable CERT_HASH is not set. It needs to be set to the thumbprint of the signing certificate."
            exit 1
        fi
    fi
}

# Get the project version (specified in Cargo.toml).
# This outputs string such as 1.0.0.
function product_version {
    sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml
}

# Run cargo with all appropriate flags and options
# Arguments:
# - (optional) target
function build_executable {
    local -a target_args=()

    if [[ -n "${1-}" ]]; then
        target_args+=(--target "$1")
    fi

    # Old bash versions complain about empty array expansion when -u is set
    set +u

    local rustflags="-C codegen-units=1 -C panic=abort -C strip=symbols -C opt-level=z"

    if [[ -z "$1" && "$(uname -s)" == "MINGW"* ]] || [[ $1 == *"windows"* ]]; then
        rustflags+=" -Ctarget-feature=+crt-static"
    fi

    RUSTFLAGS="$rustflags" cargo build --bin installer-downloader --release "${target_args[@]}"

    set -u
}

# Combine executables on macOS. This must be run after build_executable for both x86 and arm64.
function lipo_executables {
    local target_exes
    target_exes=()

    rm -rf "$BUILD_DIR/installer-downloader"

    case $HOST in
        x86_64-apple-darwin) target_exes=(
            "$CARGO_TARGET_DIR/release/installer-downloader"
            "$CARGO_TARGET_DIR/aarch64-apple-darwin/release/installer-downloader"
        )
        ;;
        aarch64-apple-darwin) target_exes=(
            "$CARGO_TARGET_DIR/release/installer-downloader"
            "$CARGO_TARGET_DIR/x86_64-apple-darwin/release/installer-downloader"
        )
        ;;
    esac

    lipo "${target_exes[@]}" -create -output "$BUILD_DIR/installer-downloader"
}

# Create temporary keychain for importing $CSC_LINK
function setup_macos_keychain {
    log_info "Creating a temporary keychain \"$SIGN_KEYCHAIN_PATH\" for $CSC_LINK"

    SIGN_KEYCHAIN_PASS=$(openssl rand -base64 64)
    export SIGN_KEYCHAIN_PASS

    delete_macos_keychain
    trap "delete_macos_keychain" EXIT

    /usr/bin/security create-keychain -p "$SIGN_KEYCHAIN_PASS" "$SIGN_KEYCHAIN_PATH"
    /usr/bin/security unlock-keychain -p "$SIGN_KEYCHAIN_PASS" "$SIGN_KEYCHAIN_PATH"
    /usr/bin/security set-keychain-settings "$SIGN_KEYCHAIN_PATH"

    # Include keychain in the search list, or codesign won't find it
    /usr/bin/security list-keychains -d user -s "$SIGN_KEYCHAIN_PATH"

    log_info "Importing PKCS #12 to keychain"

    /usr/bin/security import "$CSC_LINK" -k "$SIGN_KEYCHAIN_PATH" -P "$CSC_KEY_PASSWORD" -T /usr/bin/codesign

    # Prevent password prompt when signing
    /usr/bin/security set-key-partition-list -S "apple-tool:,apple:" -s -k "$SIGN_KEYCHAIN_PASS" "$SIGN_KEYCHAIN_PATH"

    log_info "Done."

    # Find identity
    log_info "Find the identity to use"

    /usr/bin/security find-identity -p codesigning
    read -rp "Enter identity: " SIGN_KEYCHAIN_IDENTITY
    export SIGN_KEYCHAIN_IDENTITY

    # TODO: auto-detect identity
}

function delete_macos_keychain {
    /usr/bin/security delete-keychain "$SIGN_KEYCHAIN_PATH" || true
    rm -f "$SIGN_KEYCHAIN_PATH"
}

# Sign an artifact.
# - setup_macos_keychain must be called first
# Arguments:
# - file to sign
function sign_macos {
    local file="$1"
    if [[ "$SIGN" == "false" ]]; then
        # Ad-hoc sign app bundle
        /usr/bin/codesign --identifier "$BUNDLE_ID" \
            --sign - \
            --timestamp=none --verbose=0 -o runtime \
            "$file"
    else
        /usr/bin/codesign --identifier "$BUNDLE_ID" \
            --sign "$SIGN_KEYCHAIN_IDENTITY" \
            --keychain "$SIGN_KEYCHAIN_PATH" \
            --verbose=0 -o runtime \
            "$file"
    fi
}

# Build app bundle and dmg, and optionally sign it.
# If `$SIGN` is false, the app bundle is only ad-hoc signed.
function dist_macos_app {
    local app_path="$BUILD_DIR/$FILENAME.app/"
    local dmg_path="$BUILD_DIR/$FILENAME.dmg"

    # Build app bundle
    log_info "Building $app_path..."

    rm -rf "$app_path"

    mkdir -p "$app_path/Contents/Resources"
    cp "../dist-assets/icon.icns" "$app_path/Contents/Resources/"

    mkdir -p "$app_path/Contents/MacOS"

    # Generate info plist, using the version specified in Cargo.toml
    sed -e "s/%BUNDLE_VERSION%/$(product_version)/g" \
        -e "s/%BUNDLE_NAME%/$BUNDLE_NAME/g" \
        -e "s/%BUNDLE_ID%/$BUNDLE_ID/g" \
        ./assets/Info.plist > "$app_path/Contents/Info.plist"

    # Copy executable
    cp "$BUILD_DIR/installer-downloader" "$app_path/Contents/MacOS/installer-downloader"

    # Sign app bundle
    if [[ "$SIGN" != "false" ]]; then
        setup_macos_keychain
    fi
    sign_macos "$app_path"

    # Pack in .dmg
    log_info "Creating $dmg_path image..."
    hdiutil create -volname "$FILENAME" -srcfolder "$app_path" -ov -format UDZO \
        "$dmg_path"

    # Sign .dmg
    sign_macos "$dmg_path"

    # Notarize .dmg
    if [[ "$SIGN" != "false" ]]; then
        notarize_mac "$dmg_path"
    fi

    # Move to dist dir
    log_info "Moving final artifacts to $DIST_DIR"
    rm -rf "$DIST_DIR/$FILENAME.app/"
    rm -rf "$DIST_DIR/$FILENAME.dmg"
    mv "$app_path" "$DIST_DIR/"
    mv "$dmg_path" "$DIST_DIR/"
}

# Notarize and staple a file.
# Arguments:
# - file to sign
function notarize_mac {
    local file="$1"

    log_info "Notarizing $file"
    xcrun notarytool submit "$file" \
        --keychain "$NOTARIZE_KEYCHAIN" \
        --keychain-profile "$NOTARIZE_KEYCHAIN_PROFILE" \
        --wait

    log_info "Stapling $file"
    xcrun stapler staple "$file"
}

# Sign a file.
# Arguments:
# - file to sign
function sign_win {
    local binary=$1
    local num_retries=3

    for i in $(seq 0 ${num_retries}); do
        log_info "Signing $binary..."
        if signtool sign \
            -tr http://timestamp.digicert.com -td sha256 \
            -fd sha256 -d "Mullvad VPN installer" \
            -du "https://github.com/mullvad/mullvadvpn-app#readme" \
            -sha1 "$CERT_HASH" "$binary"
        then
            break
        fi

        if [ "$i" -eq "${num_retries}" ]; then
            return 1
        fi

        sleep 1
    done
}

# Copy executable and optionally sign it.
function dist_windows_app {
    cp "$CARGO_TARGET_DIR/release/installer-downloader.exe" "$BUILD_DIR/$FILENAME.exe"
    if [[ "$SIGN" != "false" ]]; then
        sign_win "$BUILD_DIR/$FILENAME.exe"
    fi
    mv "$BUILD_DIR/$FILENAME.exe" "$DIST_DIR/"
}

# Upload whatever matches the first argument to the Linux build server
# Arguments:
# - local file
# - version
function upload_sftp {
    local local_path=$1
    local version=$2
    echo "Uploading \"$local_path\" to app-build-linux:upload/installer-downloader/$version"
    sftp app-build-linux <<EOF
mkdir upload/installer-downloader
mkdir upload/installer-downloader/$version
chmod 770 upload/installer-downloader
chmod 770 upload/installer-downloader/$version
cd upload/installer-downloader/$version
put "$local_path"
bye
EOF
}

# Upload latest build and checksum in the dist directory to Linux build server
# The artifacts MUST have been built already
# The working directory MUST be $DIST_DIR
#
# Arguments:
# - version
function upload {
    local version=$1
    local files=( "$FILENAME."* )

    local checksums_path
    checksums_path="installer-downloader+$(hostname)+$version.sha256"

    sha256sum "${files[@]}" > "$checksums_path"

    for file in "${files[@]}"; do
        upload_sftp "$file" "$version" || return 1
    done
    upload_sftp "$checksums_path" "$version" || return 1
}

# Check if the current commit has a signed tag
#
# Arguments:
# - version
function verify_version_tag {
    local version=$1

    local expect_tag="${TAG_PREFIX}${version}"
    log_info "Current commit must have tag: $expect_tag"

    local tag
    set +e
    tag=$(git describe --exact-match --tags)
    local describe_exit=$?
    set -e

    if [[ $describe_exit -ne 0 ]]; then
        log_error "'git describe' failed for the current commit (no tag?). Expected tag $expect_tag"
        exit 1
    fi

    if [[ "$tag" != "$expect_tag" ]]; then
        log_error "Unexpected tag found for current commit. Expected $expect_tag. Found: $tag"
        exit 1
    fi

    log_info "Verifying tag $tag..."
    git verify-tag "$tag"
}

function main {
    if [[ "$SIGN" != "false" ]]; then
        assert_can_sign
    fi

    if [[ "$(uname -s)" == "Darwin" ]]; then
        case $HOST in
            x86_64-apple-darwin) TARGETS=("" aarch64-apple-darwin);;
            aarch64-apple-darwin) TARGETS=("" x86_64-apple-darwin);;
        esac

        for t in "${TARGETS[@]:-"$HOST"}"; do
            build_executable "$t"
        done

        lipo_executables
        dist_macos_app

    elif [[ "$(uname -s)" == "MINGW"* ]]; then
        build_executable
        dist_windows_app
    fi

    if [[ "$UPLOAD" == "true" ]]; then
        local version
        version=$(product_version)

        verify_version_tag "$version"

        (cd "$DIST_DIR" && upload "$version") || return 1
    fi
}

main
