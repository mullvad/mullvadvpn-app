#!/usr/bin/env bash

# Script to generate merged and normalized SBOMs for our Android app. It relies on SBOMs generated
# by Gradle and Cargo.

# The jq programs passed to jq_edit are single-quoted on purpose
# shellcheck disable=SC2016

set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
REPORTS_DIR="$REPO_ROOT/android/app/build/reports"
JNI_CRATE_DIR="$REPO_ROOT/mullvad-jni"
DIST_DIR="$REPO_ROOT/dist"
CARGO_TARGETS=(aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android)
VERSION="$(cargo run -q --manifest-path "$REPO_ROOT/Cargo.toml" --bin mullvad-version versionName)"
COMMIT_TIME="$(git -C "$REPO_ROOT" log -1 --format=%ct)"
ORGANIZATION='{"name": "Mullvad VPN AB", "url": ["https://mullvad.net"]}'
# SBOM file name suffix per app flavor, matching the APK names
declare -A FLAVOR_SUFFIXES=([oss]="" [play]=".play")

# Avoid localization (ICU) in cyclonedx-cli to keep output reproducible
export DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1

function main {
    trap cleanup EXIT

    generate_android_sboms
    describe_android_sboms

    generate_cargo_sboms
    normalize_cargo_sboms
    describe_cargo_sboms

    local flavor merged_sbom_file
    for flavor in "${!FLAVOR_SUFFIXES[@]}"; do
        merged_sbom_file="$DIST_DIR/MullvadVPN-$VERSION${FLAVOR_SUFFIXES[$flavor]}.cdx.json"
        merge_sboms "$flavor" "$merged_sbom_file"
        normalize_merged_sbom "$merged_sbom_file"
        describe_merged_sbom "$merged_sbom_file"
        verify_sbom "$merged_sbom_file"
    done
}

function generate_android_sboms {
    "$REPO_ROOT/android/gradlew" -p "$REPO_ROOT/android" --console plain \
        generateOssProdReleaseSbom generatePlayProdReleaseSbom
}

function describe_android_sboms {
    local sbom_file
    for sbom_file in "$REPORTS_DIR"/*.jvm.cdx.json; do
        jq_edit "$sbom_file" --argjson mullvad "$ORGANIZATION" '
            # Set Mullvad as supplier and GPLv3 as license for our Gradle modules
            (.metadata.component, (.components[] | select(.purl // "" | contains("project_path="))))
                += {licenses: [{expression: "GPL-3.0-or-later"}], supplier: $mullvad}'
    done
}

function generate_cargo_sboms {
    local target
    for target in "${CARGO_TARGETS[@]}"; do
        cargo cyclonedx --format json --spec-version 1.5 \
            --target "$target" --target-in-filename \
            --manifest-path "$JNI_CRATE_DIR/Cargo.toml"
    done
}

function normalize_cargo_sboms {
    # IDs of our crates start with the checkout path, e.g. path+file:///build/
    local jni_crate_id local_crate_id_prefix
    jni_crate_id="$(cargo pkgid --manifest-path "$JNI_CRATE_DIR/Cargo.toml")"
    local_crate_id_prefix="$(dirname "$jni_crate_id")/"

    local target
    for target in "${CARGO_TARGETS[@]}"; do
        jq_edit "$JNI_CRATE_DIR/mullvad-jni_$target.cdx.json" \
            --argjson used_crates "$(print_jni_crates normal,build "$target")" \
            --arg local_crate_id_prefix "$local_crate_id_prefix" '
            def crate: "\(.name)@\(.version)";

            # Drop unused crates due to cargo-cyclonedx resolving all workspace features
            [.components[] | select(crate | IN($used_crates[]) | not) | ."bom-ref"]
                as $unused_crate_refs
            | del(.components[] | select(."bom-ref" | IN($unused_crate_refs[])))
            | del(.dependencies[] | select(.ref | IN($unused_crate_refs[])))
            | del(.dependencies[].dependsOn[]? | select(IN($unused_crate_refs[])))

            # Make crate IDs reproducible by making them relative, e.g.
            # path+file:///build/mullvad-api#0.0.0 becomes path+file://./mullvad-api#0.0.0
            | walk(
                if type == "string" and startswith($local_crate_id_prefix)
                then "path+file://./" + ltrimstr($local_crate_id_prefix)
                else .
                end
            )'
    done
}

function describe_cargo_sboms {
    local linked_crates
    linked_crates="$(print_jni_crates normal,no-proc-macro "${CARGO_TARGETS[@]}")"

    local sbom_file
    for sbom_file in "$JNI_CRATE_DIR"/mullvad-jni_*.cdx.json; do
        jq_edit "$sbom_file" \
            --argjson linked_crates "$linked_crates" --argjson mullvad "$ORGANIZATION" '
            def crate: "\(.name)@\(.version)";
            def workspace_crate: ."bom-ref" | startswith("path+file://");
            def mullvad_crate: workspace_crate or (.author // "" | contains("Mullvad VPN"));

            # Mark build-time dependencies that are not shipped as excluded
            .components[] |= (.scope =
                if crate | IN($linked_crates[]) then "required" else "excluded" end)

            # Set Mullvad as supplier of our own crates
            | (.metadata.component, (.components[] | select(mullvad_crate))).supplier = $mullvad'
    done
}

# Prints the crates of mullvad-jni for the given dependency kinds and targets as a JSON array of
# name@version, based on cargo tree lines like "mullvad-api v0.0.0 (/build/mullvad-api)"
function print_jni_crates {
    local edges=$1
    shift
    local target target_args=()
    for target in "$@"; do
        target_args+=(--target "$target")
    done

    local cargo_tree_output
    cargo_tree_output="$(cargo tree --locked --manifest-path "$JNI_CRATE_DIR/Cargo.toml" \
        "${target_args[@]}" --edges "$edges" --prefix none --format '{p}')"

    jq -Rn '[
        inputs
        | select(. != "")
        | split(" ") as [$name, $version]
        | "\($name)@\($version | ltrimstr("v"))"
    ] | unique' <<< "$cargo_tree_output"
}

function merge_sboms {
    local flavor=$1
    local merged_sbom_file=$2
    local gradle_sbom_file="$REPORTS_DIR/${flavor}ProdRelease.jvm.cdx.json"

    mkdir -p "$DIST_DIR"
    cyclonedx merge \
        --input-files "$gradle_sbom_file" "$JNI_CRATE_DIR"/mullvad-jni_*.cdx.json \
        --input-format json --output-format json --output-version v1_5 \
        --output-file "$merged_sbom_file"
}

function normalize_merged_sbom {
    local sbom_file=$1
    jq_edit "$sbom_file" --argjson commit_time "$COMMIT_TIME" '
        # Drop random serial to make report reproducible
        del(.serialNumber)

        # Rewrite timestamp to make it reproducible
        | .metadata.timestamp = ($commit_time | todate)

        # Drop app module duplicate in components section
        | .metadata.component."bom-ref" as $app_gradle_module_ref
        | del(.components[] | select(."bom-ref" == $app_gradle_module_ref))

        # Combine dependencies that differ on a target basis since the final binaries support
        # all targets
        | .dependencies |= (
            group_by(.ref)
            | map(. as $entries | {
                ref: $entries[0].ref,
                dependsOn: ($entries | map(.dependsOn // []) | add | unique)
            })
        )'
}

function describe_merged_sbom {
    local sbom_file=$1
    jq_edit "$sbom_file" --argjson mullvad "$ORGANIZATION" '
        # Link the main Gradle app module and the mullvad-jni crate
        .metadata.component."bom-ref" as $app_gradle_module_ref
        | [.components[] | select(."bom-ref" | startswith("path+file://./mullvad-jni#"))]
            as $jni_crates
        | if ($jni_crates | length) != 1 then error("expected one mullvad-jni crate") else . end
        | $jni_crates[0]."bom-ref" as $jni_crate_ref
        | (.dependencies[] | select(.ref == $app_gradle_module_ref)).dependsOn
            |= (. + [$jni_crate_ref] | unique)

        # Set Mullvad as authors
        | .metadata.authors = [{name: $mullvad.name}]'
}

function verify_sbom {
    local sbom_file=$1
    cyclonedx validate --input-file "$sbom_file" --input-format json --input-version v1_5 \
        --fail-on-errors
}

function jq_edit {
    local sbom_file=$1
    shift
    jq "$@" "$sbom_file" > "$sbom_file.tmp"
    mv "$sbom_file.tmp" "$sbom_file"
}

# Clean up intermediate SBOM files created by cargo-cyclonedx next to every Cargo.toml
function cleanup {
    find "$REPO_ROOT" -name '*_*-linux-android*.cdx.json' -delete
}

main
