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
DESKTOP="false"
for argument in "$@"; do
    case "$argument" in
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
        # Windows C++
        cp dist-assets/windows/version.h dist-assets/windows/version.h.bak
        cat <<EOF > dist-assets/windows/version.h
#define MAJOR_VERSION $semver_major
#define MINOR_VERSION $semver_minor
#define PATCH_VERSION $semver_patch
#define PRODUCT_VERSION "$PRODUCT_VERSION"
EOF
    fi
}

function restore_backup {
    set +e

    if [[ "$DESKTOP" == "true" ]]; then
        # Windows C++
        mv dist-assets/windows/version.h.bak dist-assets/windows/version.h

    fi
    set -e
}

function delete_backup {
    set +e

    if [[ "$DESKTOP" == "true" ]]; then
        # Windows C++
        rm dist-assets/windows/version.h.bak

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
