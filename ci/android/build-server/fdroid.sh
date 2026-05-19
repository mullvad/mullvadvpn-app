#!/usr/bin/env bash
#
# Usage:
#   fdroid.sh setup-repo <env>                 # bootstrap a single repo dir
#   fdroid.sh update-and-deploy <version-dir>  # copy artifacts, sign+update index, deploy
#   fdroid.sh deploy-only                      # re-deploy current repo state to all envs

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# shellcheck source=ci/android/build-server/config.sh
source "$SCRIPT_DIR/config.sh"

BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
cd "$BUILD_DIR"

function main {
    case "${1-}" in
        setup-repo|update-and-deploy|deploy-only)
            local subcmd=$1
            shift
            "$subcmd" "$@"
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
    cat <<EOF
Usage:
  $(basename "$0") setup-repo <env>                 bootstrap a single repo dir
  $(basename "$0") update-and-deploy <version-dir>  copy artifacts, sign+update index, deploy
  $(basename "$0") deploy-only                      re-deploy current repo state to all envs
EOF
}

function setup-repo {
    local env=${1:?Usage: setup-repo <env>}
    local fdroid_repo_root="${FDROID_REPO_DIRS[$env]:?Unknown env: $env}"
    local config_file="$SCRIPT_DIR/fdroid/config-$env.yml"
    if [[ ! -f "$config_file" ]]; then
        echo "Missing config for env '$env': $config_file" >&2
        return 1
    fi

    mkdir -p \
        "$fdroid_repo_root/repo/icons" \
        "$fdroid_repo_root/metadata/net.mullvad.mullvadvpn/en-US/images" \
        "$fdroid_repo_root/metadata/net.mullvad.mullvadvpn/en-US/changelogs"

    cp "$config_file" "$fdroid_repo_root/config.yml"
    cp "$SCRIPT_DIR/fdroid/icon.png" "$fdroid_repo_root/repo/icons/icon.png"
    cp "$SCRIPT_DIR/fdroid/net.mullvad.mullvadvpn.yml" "$fdroid_repo_root/metadata/"
    cp "$SCRIPT_DIR/fdroid/icon.png" \
        "$fdroid_repo_root/metadata/net.mullvad.mullvadvpn/en-US/images/"
}

function update-and-deploy {
    local version_dir=${1:?Usage: update-and-deploy <version-dir>}
    for env in "${!FDROID_REPO_DIRS[@]}"; do
        publish_to_env "$env" "$version_dir" || return 1
    done
    rm -rf "$version_dir"
}

function deploy-only {
    for env in "${!FDROID_REPO_DIRS[@]}"; do
        publish_to_env "$env" || return 1
    done
}

function publish_to_env {
    local env=$1
    local version_dir=${2-}
    local fdroid_repo_root="${FDROID_REPO_DIRS[$env]}"

    if [[ -n "$version_dir" ]]; then
        local apks=( "$version_dir"/MullvadVPN-*.apk )
        local changelogs=( "$version_dir"/*.txt )
        if (( ${#apks[@]} != 1 )) || (( ${#changelogs[@]} != 1 )); then
            echo "Expected exactly one apk and one changelog in $version_dir" >&2
            return 1
        fi

        cp "${apks[@]}" "$fdroid_repo_root/repo/"
        cp "${changelogs[@]}" \
            "$fdroid_repo_root/metadata/net.mullvad.mullvadvpn/en-US/changelogs/"
    fi

    local fdroid_command
    if [[ -n "$version_dir" ]]; then
        local java_opts='--add-opens=jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED'
        fdroid_command="JAVA_TOOL_OPTIONS='$java_opts' fdroid update && fdroid deploy"
    else
        fdroid_command='fdroid deploy'
    fi

    run_in_sign_container "$fdroid_repo_root" "$fdroid_command" || return 1
}

function run_in_sign_container {
    local fdroid_repo_root=$1
    local command=$2

    RCLONE_CONFIG_HOST_PATH=$RCLONE_CONFIG_PATH \
    YUBIKEY_PIN=$YUBIKEY_PIN \
    YUBIKEY_PATH=$(readlink -f /dev/android-jks-signing-key) \
    "./android/scripts/containerized-sign.sh" "$fdroid_repo_root" "$command"
}

main "$@"
