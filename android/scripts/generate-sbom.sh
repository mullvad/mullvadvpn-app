#!/usr/bin/env bash
#
# Merges the per-flavor JVM SBOMs (from Gradle's generate{Oss,Play}ProdReleaseSbom)
# with the native Rust deps of libmullvad_jni.so into
# dist/MullvadVPN-<version>[.play].sbom.cdx.json.
#
# Requires cargo-cyclonedx, cyclonedx (cyclonedx-cli) and jq (Android build container).

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
REPORTS_DIR="$REPO_ROOT/android/app/build/reports"
DIST_DIR="$REPO_ROOT/dist"
JNI_MANIFEST="$REPO_ROOT/mullvad-jni/Cargo.toml"

jvm_sboms=("$REPORTS_DIR"/*.jvm.sbom.cdx.json)
if [[ ${#jvm_sboms[@]} -eq 0 ]]; then
    echo "No JVM SBOMs in $REPORTS_DIR. Run './gradlew -p android generate{Oss,Play}ProdReleaseSbom' first." >&2
    exit 1
fi

version="$(jq -r '.metadata.component.version' "${jvm_sboms[0]}")"
if [[ -z "$version" || "$version" == "null" ]]; then
    echo "Could not read version from ${jvm_sboms[0]}" >&2
    exit 1
fi

# Native Rust SBOM, pinned to CycloneDX 1.5 (cargo-cyclonedx has no 1.6 support).
rust_sbom="$REPORTS_DIR/rust-jni.sbom.cdx.json"
cargo cyclonedx --format json --spec-version 1.5 \
    --target aarch64-linux-android \
    --manifest-path "$JNI_MANIFEST" \
    --override-filename rust-jni.sbom.cdx
# cargo-cyclonedx writes beside the manifest; relocate it into reports.
mv "$REPO_ROOT/mullvad-jni/rust-jni.sbom.cdx.json" "$rust_sbom"

mkdir -p "$DIST_DIR"
for jvm_sbom in "${jvm_sboms[@]}"; do
    case "$(basename "$jvm_sbom")" in
        ossProdRelease.*) suffix="" ;;
        playProdRelease.*) suffix=".play" ;;
        *) echo "Unknown SBOM variant: $jvm_sbom" >&2; exit 1 ;;
    esac

    out="$DIST_DIR/MullvadVPN-$version$suffix.sbom.cdx.json"

    cyclonedx merge \
        --input-files "$jvm_sbom" "$rust_sbom" \
        --input-format json --output-format json --output-version v1_5 \
        --name mullvad-vpn-android --version "$version" \
        --output-file "$out"

    # Drop optional volatile fields for byte-reproducibility.
    jq 'del(.serialNumber, .metadata.timestamp)' "$out" > "$out.tmp"
    mv "$out.tmp" "$out"
done
