#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
REPORTS_DIR="$REPO_ROOT/android/app/build/reports"
DIST_DIR="$REPO_ROOT/dist"
CARGO_SBOM_NAME="rust-jni.cdx.json"
CARGO_SBOM="$REPORTS_DIR/$CARGO_SBOM_NAME"

export DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1

function main {
    trap cleanup EXIT

    local version
    version="$(cargo run -q --manifest-path "$REPO_ROOT/Cargo.toml" --bin mullvad-version versionName)"

    generate_android_sboms
    generate_cargo_sbom

    merge ossProdRelease "$version" "MullvadVPN-$version.apk.cdx.json"
    merge playProdRelease "$version" "MullvadVPN-$version.play.apk.cdx.json"
}

function generate_android_sboms {
    "$REPO_ROOT/android/gradlew" -p "$REPO_ROOT/android" --console plain \
        generateOssProdReleaseSbom generatePlayProdReleaseSbom
}

function generate_cargo_sbom {
    cargo cyclonedx --format json --spec-version 1.5 \
        --target aarch64-linux-android \
        --manifest-path "$REPO_ROOT/mullvad-jni/Cargo.toml" \
        --override-filename "${CARGO_SBOM_NAME%.json}"
    mv "$REPO_ROOT/mullvad-jni/$CARGO_SBOM_NAME" "$CARGO_SBOM"
    sed -i "s|file://$REPO_ROOT/|file://./|g" "$CARGO_SBOM"
}

function merge {
    local variant=$1
    local version=$2
    local out="$DIST_DIR/$3"
    local gradle_sbom="$REPORTS_DIR/$variant.jvm.cdx.json"

    mkdir -p "$DIST_DIR"
    cyclonedx merge \
        --input-files "$gradle_sbom" "$CARGO_SBOM" \
        --input-format json --output-format json --output-version v1_5 \
        --name mullvad-vpn-android --version "$version" \
        | jq 'del(.serialNumber, .metadata.timestamp)' > "$out"
}

function cleanup {
    find "$REPO_ROOT" -name "$CARGO_SBOM_NAME" -not -path "$CARGO_SBOM" -delete
}

main
