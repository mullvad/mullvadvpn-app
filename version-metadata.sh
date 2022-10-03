#!/usr/bin/env bash
#
# Can inject correctly formatted version strings/numbers in all the various
# project metadata files. Can also back them up and restore them.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Parse arguments
COMMAND="$1"
shift 1

PRODUCT_VERSION=""
ANDROID="false"
DESKTOP="false"
for argument in "$@"; do
    case "$argument" in
        "--android")
            ANDROID="true"
            ;;
        "--desktop")
            DESKTOP="true"
            ;;
        -*)
            echo >&2 "Unknown option \"$argument\""
            exit 1
            ;;
        *)
            PRODUCT_VERSION="$argument"
            ;;
    esac
done

function inject_version {
    # Regex that only matches valid Mullvad VPN versions. It also captures
    # relevant values into capture groups, read out via BASH_REMATCH[x].
    local VERSION_REGEX="^20([0-9]{2})\.([1-9][0-9]?)(-beta([1-9][0-9]?))?(-dev-[0-9a-f]+)?$"

    if [[ ! $PRODUCT_VERSION =~ $VERSION_REGEX ]]; then
        echo >&2 "Invalid version format. Please specify version as:"
        echo >&2 "<YEAR>.<NUMBER>[-beta<NUMBER>]"
        return 1
    fi

    local semver_version
    semver_version=$(echo "$PRODUCT_VERSION" | sed -Ee 's/($|-.*)/.0\1/g')
    local semver_major="20${BASH_REMATCH[1]}"
    local semver_minor=${BASH_REMATCH[2]}
    local semver_patch="0"

    if [[ "$DESKTOP" == "true" ]]; then
        echo "Setting desktop version to $semver_version"

        # Electron GUI
        cp gui/package.json gui/package.json.bak
        cp gui/package-lock.json gui/package-lock.json.bak
        (cd gui/ && npm version "$semver_version" --no-git-tag-version --allow-same-version)

        # Windows C++
        cp dist-assets/windows/version.h dist-assets/windows/version.h.bak
        cat <<EOF > dist-assets/windows/version.h
#define MAJOR_VERSION $semver_major
#define MINOR_VERSION $semver_minor
#define PATCH_VERSION $semver_patch
#define PRODUCT_VERSION "$PRODUCT_VERSION"
EOF
    fi

    if [[ "$ANDROID" == "true" ]]; then
        # Android
        local version_year
        version_year=$(printf "%02d" "${BASH_REMATCH[1]}")
        local version_number
        version_number=$(printf "%02d" "${BASH_REMATCH[2]}")
        local version_patch="00" # Not used for now.
        local version_beta
        version_beta=$(printf "%02d" "${BASH_REMATCH[4]:-99}")
        local android_version_code=${version_year}${version_number}${version_patch}${version_beta}

        echo "Setting Android versionName to $PRODUCT_VERSION and versionCode to $android_version_code"

        cp android/app/build.gradle.kts android/app/build.gradle.kts.bak
        sed -i -Ee "s/versionCode = [0-9]+/versionCode = $android_version_code/g" \
            android/app/build.gradle.kts
        sed -i -Ee "s/versionName = \"[^\"]+\"/versionName = \"$PRODUCT_VERSION\"/g" \
            android/app/build.gradle.kts
    fi
}

function restore_backup {
    set +e

    if [[ "$DESKTOP" == "true" ]]; then
        # Electron GUI
        mv gui/package.json.bak gui/package.json
        mv gui/package-lock.json.bak gui/package-lock.json
        # Windows C++
        mv dist-assets/windows/version.h.bak dist-assets/windows/version.h

    fi

    if [[ "$ANDROID" == "true" ]]; then
        # Android
        mv android/app/build.gradle.kts.bak android/app/build.gradle.kts
    fi
    set -e
}

function delete_backup {
    set +e

    if [[ "$DESKTOP" == "true" ]]; then
        # Electron GUI
        rm gui/package.json.bak
        rm gui/package-lock.json.bak
        # Windows C++
        rm dist-assets/windows/version.h.bak

    fi

    if [[ "$ANDROID" == "true" ]]; then
        # Android
        rm android/app/build.gradle.kts.bak
    fi
    set -e
}

case "$COMMAND" in
    "inject")
        inject_version
        ;;
    "restore-backup")
        restore_backup
        ;;
    "delete-backup")
        delete_backup
        ;;
    *)
        echo >&2 "Invalid command"
        exit 1
        ;;
esac
