#!/usr/bin/env bash

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# shellcheck source=ci/android/build-server/config.sh
source "$SCRIPT_DIR/config.sh"

BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
cd "$BUILD_DIR"

function main {
    case "${1-}" in
        setup-repo)
            shift
            setup-repo "$@"
            ;;
        publish)
            shift
            local version_dir=${1-}
            [[ -n "$version_dir" ]] && { for_each_repo stage "$version_dir" || return 1; }
            for_each_repo update_and_deploy || return 1
            [[ -n "$version_dir" ]] && rm -rf "$version_dir"
            ;;
        "")
            usage >&2
            return 1
            ;;
        *)
            echo "Unknown subcommand: $1" >&2
            usage >&2
            return 1
            ;;
    esac
}

function usage {
    echo "Usage: $0 <subcommand> [args]"
    echo
    echo "Subcommands:"
    echo "  setup-repo <repo-env>  bootstrap a single repo dir"
    echo "  publish [version-dir]  stage first if version provided, then update and deploy all repos"
}

function setup-repo {
    local repo_env=${1:?Usage: setup-repo <repo-env>}
    local config_src="$BUILD_DIR/ci/android/build-server/fdroid/config.$repo_env.yml"
    [[ -f "$config_src" ]] || { echo "Missing config source for: $repo_env" >&2; return 1; }
    local repo_dir="$FDROID_REPOS_DIR/$repo_env"

    mkdir -p \
        "$repo_dir/repo/icons" \
        "$repo_dir/metadata/net.mullvad.mullvadvpn/en-US/images" \
        "$repo_dir/metadata/net.mullvad.mullvadvpn/en-US/changelogs"

    cp "$config_src" "$repo_dir/config.yml"

    local icon_src="$BUILD_DIR/android/src/main/play/listings/en-US/graphics/icon/icon.png"
    cp "$icon_src" "$repo_dir/repo/icons/icon.png"
    cp "$icon_src" "$repo_dir/metadata/net.mullvad.mullvadvpn/en-US/images/icon.png"

    local metadata_dest="$repo_dir/metadata/net.mullvad.mullvadvpn.yml"
    # Avoid overwriting existing metadata since it contains state (CurrentVersion/CurrentVersionCode).
    if [[ -e "$metadata_dest" ]]; then
        echo "Metadata already present at $metadata_dest, leaving it alone."
    else
        cp "$BUILD_DIR/ci/android/build-server/fdroid/net.mullvad.mullvadvpn.yml" \
            "$metadata_dest"
    fi
}

function for_each_repo {
    local command=$1; shift
    local repo_dirs=( "$FDROID_REPOS_DIR"/*/ )
    if (( ${#repo_dirs[@]} == 0 )); then
        echo "No repos set up. Run setup-repo first." >&2
        return 1
    fi
    for repo_dir in "${repo_dirs[@]}"; do
        "$command" "$repo_dir" "$@" || return 1
    done
}

function stage {
    local repo_dir=$1
    local version_dir=$2

    local apks=( "$version_dir"/MullvadVPN-*.apk )
    local changelogs=( "$version_dir"/*.txt )
    if (( ${#apks[@]} != 1 )) || (( ${#changelogs[@]} != 1 )); then
        echo "Expected exactly one apk and one changelog in $version_dir" >&2
        return 1
    fi

    cp "${apks[@]}" "$repo_dir/repo/"
    cp "${changelogs[@]}" \
        "$repo_dir/metadata/net.mullvad.mullvadvpn/en-US/changelogs/"

    local version
    version=$(basename "$version_dir")
    local version_code
    version_code=$(basename "${changelogs[0]}" .txt)

    # Bump current/suggested version only for stable builds.
    if [[ "$version" != *"-alpha"* && "$version" != *"-beta"* && "$version" != *"-dev-"* ]]; then
        sed -i -E \
            -e "s/^CurrentVersion: .*/CurrentVersion: $version/" \
            -e "s/^CurrentVersionCode: .*/CurrentVersionCode: $version_code/" \
            "$repo_dir/metadata/net.mullvad.mullvadvpn.yml"
    fi
}

function update_and_deploy {
    local repo_dir=$1
    local java_opts='--add-opens=jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED'
    run_in_sign_container "$repo_dir" "JAVA_TOOL_OPTIONS='$java_opts' fdroid update && fdroid deploy"
}

function run_in_sign_container {
    local repo_dir=$1
    local command=$2

    RCLONE_CONFIG_HOST_PATH="$FDROID_RCLONE_CONFIG_PATH" \
    YUBIKEY_PIN=$YUBIKEY_PIN \
    YUBIKEY_PATH=$(readlink -f /dev/android-jks-signing-key) \
    "$BUILD_DIR/android/scripts/containerized-sign.sh" "$repo_dir" "$command"
}

main "$@"
