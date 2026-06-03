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
            local repo_env=${1:?Usage: setup-repo <repo-env>}
            setup-repo "$repo_env" || return 1
            ;;
        stage)
            shift
            if (( $# != 3 )); then
                echo "Usage: stage <repo-env> <version> <artifact-dir>" >&2
                return 1
            fi
            local repo_env=$1
            local version=$2
            local artifact_dir=$3
            stage "$repo_env" "$version" "$artifact_dir" || return 1
            ;;
        publish)
            shift
            local repo_env=${1:?Usage: publish <repo-env>}
            local repo_dir="$FDROID_REPOS_DIR/$repo_env"
            [[ -d "$repo_dir" ]] || { echo "Unknown repo-env: $repo_env" >&2; return 1; }
            if [[ -z ${YUBIKEY_PIN-} ]]; then
                read -rsp "YUBIKEY_PIN = " YUBIKEY_PIN
                echo ""
                export YUBIKEY_PIN
            fi
            update_and_deploy "$repo_dir" || return 1
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
    echo "  setup-repo <repo-env>                       bootstrap a single repo dir"
    echo "  stage <repo-env> <version> <artifact-dir>   stage an APK + changelog into repo-env"
    echo "  publish <repo-env>                          update+deploy repo-env"
}

function setup-repo {
    local repo_env=$1
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

function stage {
    local repo_env=$1
    local version=$2
    local artifact_dir=$3
    local repo_dir="$FDROID_REPOS_DIR/$repo_env"
    [[ -d "$repo_dir" ]] || { echo "Unknown repo-env: $repo_env" >&2; return 1; }

    # Assume the single .txt in the artifact dir is the F-Droid changelog (named <versionCode>.txt at build time).
    local changelogs=( "$artifact_dir"/*.txt )
    (( ${#changelogs[@]} == 1 )) || { echo "Expected exactly one changelog in $artifact_dir" >&2; return 1; }

    cp "$artifact_dir/MullvadVPN-$version.apk" "$repo_dir/repo/"
    cp "${changelogs[@]}" "$repo_dir/metadata/net.mullvad.mullvadvpn/en-US/changelogs/"

    # Bump current/suggested version only for stable builds.
    if [[ "$version" != *"-alpha"* && "$version" != *"-beta"* && "$version" != *"-dev-"* ]]; then
        sed -i -E \
            -e "s/^CurrentVersion: .*/CurrentVersion: $version/" \
            -e "s/^CurrentVersionCode: .*/CurrentVersionCode: $(basename "${changelogs[0]}" .txt)/" \
            "$repo_dir/metadata/net.mullvad.mullvadvpn.yml"
    fi
}

function update_and_deploy {
    local repo_dir=$1
    local java_opts='--add-opens=jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED'

    RCLONE_CONFIG_HOST_PATH="$FDROID_RCLONE_CONFIG_PATH" \
    YUBIKEY_PIN=$YUBIKEY_PIN \
    YUBIKEY_PATH=$(readlink -f /dev/android-jks-signing-key) \
    "$BUILD_DIR/android/scripts/containerized-sign.sh" "$repo_dir" \
        "JAVA_TOOL_OPTIONS='$java_opts' fdroid update && fdroid deploy"
}

main "$@"
