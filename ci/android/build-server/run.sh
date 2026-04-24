#!/usr/bin/env bash

# # Setup instructions before this script will work
#
# * Follow the instructions in ../README.md
# * Import and trust the GPG keys of everyone who the build server should trust code from
# * Ensure that the machine running this script is allowed to upload to releases.mullvad.net.

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
LAST_BUILT_DIR="$SCRIPT_DIR/last-built"
UPLOAD_DIR="/home/upload/upload"
PLAY_CREDENTIALS_PATH="$SCRIPT_DIR/credentials-android/play-api-key.json"

BRANCHES_TO_BUILD=("origin/main")
TAG_PATTERN_TO_BUILD="^android/"

SPECIFIC_REF=""
ENABLE_GPG_VERIFICATION="true"
ENABLE_SIGNING="true"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --ref) SPECIFIC_REF="$2"; shift 2 ;;
        --no-verify) ENABLE_GPG_VERIFICATION="false"; shift ;;
        --no-sign) ENABLE_SIGNING="false"; shift ;;
        --upload-dir) UPLOAD_DIR="$2"; shift 2 ;;
        *) echo "Unknown argument: $1"; exit 1 ;;
    esac
done

if [[ ! -d "$UPLOAD_DIR" ]]; then
    echo "Upload directory does not exist: $UPLOAD_DIR"
    exit 1
fi

if [[ "$ENABLE_SIGNING" == "true" && -z ${YUBIKEY_PIN-} ]]; then
    read -rsp "YUBIKEY_PIN = " YUBIKEY_PIN
    echo ""
    export YUBIKEY_PIN
fi

# Move files for CDN upload by: buildserver-upload.sh
function prepare_for_cdn_upload {
    version=$1

    # Only include files. Skip subdirectories.
    files=()
    for f in *; do
        [[ -f "$f" ]] && files+=("$f")
    done
    checksums_path="android+$(hostname)+$version.sha256"
    sha256sum "${files[@]}" > "$checksums_path"

    cp "${files[@]}" "$checksums_path" "$UPLOAD_DIR/"
}

function run_in_linux_container {
    ./building/container-run.sh linux "$@"
}

# Builds the app artifacts and move them to the passed in `artifact_dir`.
# Must pass `artifact_dir` to show where to move the built artifacts.
function build {
    if [[ -f "android/build.sh" ]]; then
        CARGO_TARGET_VOLUME_NAME="cargo-target-android" \
        CARGO_REGISTRY_VOLUME_NAME="cargo-registry-android" \
        ./building/containerized-build.sh android --app-bundle || return 1
    else
        CARGO_TARGET_VOLUME_NAME="cargo-target-android" \
        CARGO_REGISTRY_VOLUME_NAME="cargo-registry-android" \
        ./building/containerized-build.sh android-gradle fullRelease || return 1
    fi

    mv dist/*.{aab,apk} "$artifact_dir" || return 1
}

# Checks out the passed git reference passed to the working directory.
# Returns an error code if the commit/tag at `ref` is not properly signed.
function checkout_ref {
    ref=$1
    if [[ "$ENABLE_GPG_VERIFICATION" == "true" ]]; then
        if [[ $ref == "refs/tags/"* ]] && ! git verify-tag "$ref"; then
            echo "!!!"
            echo "[#] $ref is a tag, but it failed GPG verification!"
            echo "!!!"
            return 1
        elif [[ $ref == "refs/remotes/"* ]] && ! git verify-commit "$current_hash"; then
            echo "!!!"
            echo "[#] $ref is a branch, but it failed GPG verification!"
            echo "!!!"
            return 1
        fi
    else
        echo "WARNING: GPG verification skipped for $ref"
    fi

    # Clean our working dir and check out the code we want to build
    rm -r dist/ 2&>/dev/null || true
    git reset --hard
    git checkout "$ref"
    git submodule update
    git clean -df
}

function build_sign_and_publish_ref {
    ref=$1
    tag=${2:-""}

    current_hash="$(git rev-parse "$ref^{commit}")"
    if [ -f "$LAST_BUILT_DIR/$current_hash" ]; then
        # This commit has already been built
        return 0
    fi

    echo ""
    echo "[#] $ref: $current_hash, building new packages."
    echo ""

    checkout_ref "$ref" || return 1

    # podman appends a trailing carriage return to the output. So we use `tr` to strip it
    local version=""
    version="$(run_in_linux_container 'stty -echo && cargo run -q --bin mullvad-version versionName' | tr -d "\r" || return 1)"

    local artifact_dir="dist/$version"
    mkdir -p "$artifact_dir"

    echo "Building Android app"
    artifact_dir=$artifact_dir build || return 1

    # If there is a tag for this commit then we append that to the produced artifacts
    # A version suffix should only be created if there is a tag for this commit and it is not a release build
    if [[ -n "$tag" && $version == *"-dev-"* ]]; then
        # Replace disallowed version characters in the tag with hyphens
        version_suffix="+${tag//[^0-9a-z_-]/-}"
        # Will only match paths that include *-dev-* which means release builds will not be included
        # Pipes all matching names and their new name to mv
        pushd "$artifact_dir"
        for original_file in MullvadVPN-*{.apk,.aab}; do
            new_file=$(echo "$original_file" | sed -nE "s/^(MullvadVPN-$version)(.*\.apk|.*\.aab)$/\1$version_suffix\2/p")
            mv "$original_file" "$new_file"
        done
        popd

        version="$version$version_suffix"
    fi

    # Sign all artifacts
    if [[ "$ENABLE_SIGNING" == "true" ]]; then
        YUBIKEY_PIN=$YUBIKEY_PIN \
        YUBIKEY_PATH=$(readlink -f /dev/android-jks-signing-key) \
        "./android/scripts/containerized-sign.sh" "$BUILD_DIR/$artifact_dir" || return 1
    else
        echo "WARNING: Signing skipped for $version"
    fi

    (cd "$artifact_dir" && prepare_for_cdn_upload "$version") || return 1

    touch "$LAST_BUILT_DIR/$current_hash"

    PLAY_CREDENTIALS_PATH="$PLAY_CREDENTIALS_PATH" \
    "$SCRIPT_DIR/upload-play.sh" "$artifact_dir" "$version" || echo "Failed to upload bundle $version"

    # shellcheck disable=SC2216
    yes | rm -r "$artifact_dir"

    echo ""
    echo "Successfully finished building $version at $(date)"
    echo ""
}

cd "$BUILD_DIR"

while true; do
    # Delete all tags. So when fetching we only get the ones existing on the remote
    git tag | xargs git tag -d > /dev/null

    git fetch --prune --tags 2> /dev/null || continue

    if [[ -n "$SPECIFIC_REF" ]]; then
        build_sign_and_publish_ref "$SPECIFIC_REF" || echo "Failed to build $SPECIFIC_REF"
        exit 0
    fi

    # Only build android/* tags.
    # Tags can't include spaces so SC2207 isn't a problem here
    # shellcheck disable=SC2207
    tags=( $(git tag | grep "$TAG_PATTERN_TO_BUILD") )

    for tag in "${tags[@]}"; do
        build_sign_and_publish_ref "refs/tags/$tag" "$tag" || echo "Failed to build tag $tag"
    done

    for branch in "${BRANCHES_TO_BUILD[@]}"; do
        build_sign_and_publish_ref "refs/remotes/$branch" || echo "Failed to build branch $branch"
    done

    sleep 240
done
