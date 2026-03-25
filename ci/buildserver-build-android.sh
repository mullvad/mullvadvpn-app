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
ANDROID_CREDENTIALS_DIR="$SCRIPT_DIR/credentials-android"
APKSIGNER_CMD="${APKSIGNER_CMD:-apksigner}"
SIGNING_CERTIFICATE_LINEAGE="$BUILD_DIR/ci/android-signing-config/SigningCertificateLineage"
PROVIDER_ARG="$BUILD_DIR/ci/android-signing-config/provider-arg.cfg"
KEY_ALIAS="Certificate for PIV Authentication"

BRANCHES_TO_BUILD=("origin/main")
TAG_PATTERN_TO_BUILD="^android/"

if [[ -z ${YUBIKEY_PIN-} ]]; then
    read -rsp "YUBIKEY_PIN = " YUBIKEY_PIN
    echo ""
    export YUBIKEY_PIN
fi

function upload {
    version=$1

    files=( * )
    checksums_path="android+$(hostname)+$version.sha256"
    sha256sum "${files[@]}" > "$checksums_path"

    mv "${files[@]}" "$checksums_path" "$UPLOAD_DIR/"
}

function run_in_linux_container {
    ./building/container-run.sh linux "$@"
}

# Builds the app artifacts and move them to the passed in `artifact_dir`.
# Must pass `artifact_dir` to show where to move the built artifacts.
function build {
    ANDROID_CREDENTIALS_DIR=$ANDROID_CREDENTIALS_DIR \
        CARGO_TARGET_VOLUME_NAME="cargo-target-android" \
        CARGO_REGISTRY_VOLUME_NAME="cargo-registry-android" \
        ./building/containerized-build.sh android --app-bundle || return 1

    mv dist/*.{aab,apk} "$artifact_dir" || return 1
}

# Checks out the passed git reference passed to the working directory.
# Returns an error code if the commit/tag at `ref` is not properly signed.
function checkout_ref {
    ref=$1
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

    # Clean our working dir and check out the code we want to build
    rm -r dist/ 2&>/dev/null || true
    git reset --hard
    git checkout "$ref"
    git submodule update
    git clean -df
}

function build_ref {
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

    sign_artifacts "$artifact_dir"

    # Upload files to google play, this needs to be done after the artifacts have been signed
    # Due to to the upload task only being able to upload all files in a folder we need to copy
    # the file to a specific folder every time
    local play_upload_dir="$artifact_dir/play_upload"
    mkdir -p "$play_upload_dir"
    if [[ "$version" != *"-dev-"* ]]; then
            upload_google_play "publishPlayProdReleaseBundle" "MullvadVPN-$version.play.aab" "$play_upload_dir"
        if [[ "$version" == *"-alpha"* ]]; then
            upload_google_play "publishPlayDevmoleReleaseBundle" "MullvadVPN-$version.play.devmole.aab" "$play_upload_dir"
            upload_google_play "publishPlayStagemoleReleaseBundle" "MullvadVPN-$version.play.stagemole.aab" "$play_upload_dir"
        fi
    fi
    rm -rf "$play_upload_dir"

    (cd "$artifact_dir" && upload "$version") || return 1
    # shellcheck disable=SC2216
    yes | rm -r "$artifact_dir"

    touch "$LAST_BUILT_DIR/$current_hash"

    echo ""
    echo "Successfully finished building $version at $(date)"
    echo ""
}

function sign_artifacts {
    dir=$1

    pushd "$dir"
    # Sign all apk files with the old and new key
    for apk in MullvadVPN-*.apk; do
        echo "$YUBIKEY_PIN" | $APKSIGNER_CMD -J-add-exports="jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" sign \
        --ks "$ANDROID_CREDENTIALS_DIR/app-keys.jks" \
        --ks-pass "file:$ANDROID_CREDENTIALS_DIR/keystore.properties.new" \
        --next-signer --ks NONE --ks-type PKCS11 --ks-key-alias "$KEY_ALIAS" \
        --provider-class sun.security.pkcs11.SunPKCS11 --provider-arg "$PROVIDER_ARG" \
        --lineage "$SIGNING_CERTIFICATE_LINEAGE" --rotation-min-sdk-version 28 --in "$apk"
    done

    # Sign all aab files with the upload key (new key)
    for aab in MullvadVPN-*.aab
    do
        echo "$YUBIKEY_PIN" | $APKSIGNER_CMD --J-add-exports="jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" sign \
        --ks NONE --ks-type PKCS11 --ks-key-alias "$KEY_ALIAS" \
        --provider-class sun.security.pkcs11.SunPKCS11 --provider-arg "$PROVIDER_ARG" \
        --in "$aab"
    done
    popd
}

function upload_google_play {
    task=$1
    file=$2
    upload_dir=$3

    rm -r "${upload_dir:?}/*"
    cp "$file" "$upload_dir/"

    PLAY_CREDENTIALS_PATH="$ANDROID_CREDENTIALS_DIR/play-api-key.json" \
    ./building/container-run.sh android ./android/gradlew -p android "$task" --artifact-dir "$upload_dir"
}

cd "$BUILD_DIR"

while true; do
    # Delete all tags. So when fetching we only get the ones existing on the remote
    git tag | xargs git tag -d > /dev/null

    git fetch --prune --tags 2> /dev/null || continue

    # Only build android/* tags.
    # Tags can't include spaces so SC2207 isn't a problem here
    # shellcheck disable=SC2207
    tags=( $(git tag | grep "$TAG_PATTERN_TO_BUILD") )

    for tag in "${tags[@]}"; do
        build_ref "refs/tags/$tag" "$tag" || echo "Failed to build tag $tag"
    done

    for branch in "${BRANCHES_TO_BUILD[@]}"; do
        build_ref "refs/remotes/$branch" || echo "Failed to build branch $tag"
    done

    sleep 240
done
