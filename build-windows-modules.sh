#!/usr/bin/env bash

set -eu

# List of solution configurations to build.
# Default configurations generated by Visual Studio are "Release" and "Debug".
CPP_BUILD_MODES=${CPP_BUILD_MODES:-"Debug"}
# List of target platforms to build for.
# Common platforms include "x86" and "x64".
CPP_BUILD_TARGETS=${CPP_BUILD_TARGETS:-"x64"}

IS_RELEASE=${IS_RELEASE:-"false"}

function restore_metadata_backups {
    mv dist-assets/windows/version.h.bak dist-assets/windows/version.h
}
trap 'restore_metadata_backups' EXIT

function inject_version {
    # Regex that only matches valid Mullvad VPN versions. It also captures
    # relevant values into capture groups, read out via BASH_REMATCH[x].
    local VERSION_REGEX="^20([0-9]{2})\.([1-9][0-9]?)(-beta([1-9][0-9]?))?(-dev-[0-9a-f]+)?$"

    local product_version=$(cargo run -q --bin mullvad-version)
    if [[ ! $product_version =~ $VERSION_REGEX ]]; then
        echo >&2 "Invalid version format. Please specify version as:"
        echo >&2 "<YEAR>.<NUMBER>[-beta<NUMBER>]"
        return 1
    fi

    local semver_major="20${BASH_REMATCH[1]}"
    local semver_minor=${BASH_REMATCH[2]}
    local semver_patch="0"

    cp dist-assets/windows/version.h dist-assets/windows/version.h.bak
    cat <<EOF > dist-assets/windows/version.h
#define MAJOR_VERSION $semver_major
#define MINOR_VERSION $semver_minor
#define PATCH_VERSION $semver_patch
#define PRODUCT_VERSION "$product_version"
EOF
}

function clean_solution {
    local path="$1"

    if [[ "$IS_RELEASE" == "true" ]]; then
        # Clean all intermediate and output files
        rm -r "${path:?}/bin/"* || true
    else
        echo "Will NOT clean intermediate files in $path/bin/ in dev builds"
    fi
}

function build_solution_config {
    local sln="$1"
    local config="$2"
    local platform="$3"

    set -x
    cmd.exe "/c msbuild.exe /m $(to_win_path "$sln") /p:Configuration=$config /p:Platform=$platform"
    set +x
}

# Builds visual studio solution in all (specified) configurations
function build_solution {
    local path="$1"
    local sln="$1/$2"

    clean_solution "$path"

    for mode in $CPP_BUILD_MODES; do
        for target in $CPP_BUILD_TARGETS; do
            build_solution_config "$sln" "$mode" "$target"
        done
    done
}

function to_win_path {
    local unixpath="$1"
    # if it's a relative path and starts with a dot (.), don't transform the
    # drive prefix (/c/ -> C:\)
    if echo "$unixpath" | grep '^\.' >/dev/null; then
        echo "$unixpath" | sed -e 's/^\///' -e 's/\//\\/g'
    # if it's an absolute path, transform the drive prefix
    else
        # remove the cygrdive prefix if it's there
        unixpath=$(echo "$unixpath" | sed -e 's/^\/cygdrive//')
        echo "$unixpath" | sed -e 's/^\///' -e 's/\//\\/g' -e 's/^./\0:/'
    fi
}

function get_solution_output_path {
    local solution_root="$1"
    local build_target="$2"
    local build_mode="$3"

    case $build_target in
        "x86") echo "$solution_root/bin/Win32-$build_mode";;
        "x64") echo "$solution_root/bin/x64-$build_mode";;
        *)
            echo "Unkown build target: $build_target"
            exit 1
            ;;
    esac
}

function build_nsis_plugins {
    local nsis_root_path=${CPP_ROOT_PATH:-"./windows/nsis-plugins"}

    clean_solution "$nsis_root_path"
    build_solution_config "$nsis_root_path/nsis-plugins.sln" "Release" "x86"
}

function main {
    inject_version

    local winfw_root_path=${CPP_ROOT_PATH:-"./windows/winfw"}
    local winnet_root_path=${CPP_ROOT_PATH:-"./windows/winnet"}

    build_solution "$winfw_root_path" "winfw.sln"
    build_solution "$winnet_root_path" "winnet.sln"

    local driverlogic_root_path=${CPP_ROOT_PATH:-"./windows/driverlogic"}
    build_solution "$driverlogic_root_path" "driverlogic.sln"

    build_nsis_plugins
}

main