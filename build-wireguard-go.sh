#!/usr/bin/env bash

# This script is used to build wireguard-go libraries for all the platforms.

set -eu

function is_android_build {

    for arg in "$@"
    do
        case "$arg" in
            "--android")
                return 0
        esac
    done
    return 1
}


function win_deduce_lib_executable_path {
    msbuild_path="$(which msbuild.exe)"
    msbuild_dir_path=$(dirname "$msbuild_path")
    find "$msbuild_dir_path/../../../../" -name "lib.exe" | \
        grep -i "hostx64/x64" | \
        head -n1
}

function win_gather_export_symbols {
   grep -Eo "\/\/export \w+" libwg.go | cut -d' ' -f2
}

function win_create_lib_file {
    echo "LIBRARY libwg" > exports.def
    echo "EXPORTS" >> exports.def

    for symbol in $(win_gather_export_symbols)
    do
        printf "\t$symbol\n" >> exports.def
    done

    "$(win_deduce_lib_executable_path)" \
        "/def:exports.def" \
        "/out:libwg.lib" \
        "/machine:X64"

}

function build_windows {
    echo "Building wireguard-go for Windows"
    pushd wireguard-go-windows;
        go build -v -o libwg.dll -buildmode c-shared
        win_create_lib_file

        target_dir=../build/lib/x86_64-pc-windows-msvc/
        mkdir -p $target_dir
        cp libwg.dll libwg.lib $target_dir
    popd
}

function unix_target_dir {
    local platform="$(uname -s)"
    if [[ ("${platform}" == "Linux") ]]; then
        echo "x86_64-unknown-linux-gnu"
    elif [[ ("${platform}" == "Darwin") ]]; then
        echo "x86_64-apple-darwin"
    else
        echo "Can't deduce target dir for $platform"
        return 1
    fi
}


function build_unix {
    echo "Building wireguard-go for $1"
    pushd wireguard-go;
        go build -v -o libwg.a -buildmode c-archive
        target_dir="../build/lib/$(unix_target_dir)"
        mkdir -p $target_dir
        cp libwg.a $target_dir
    popd
}

function build_android {
        echo Building for android
        local docker_image_hash="d73fdea1108cd75d7eb09f8894fe6892dc502a2d62c39b4f75072e777398f477"

        docker run --rm \
            -v $(pwd):/workspace \
            -w /workspace/wireguard-go \
            --entrypoint "/workspace/wireguard-go/build-android.sh" \
            mullvadvpn/mullvad-android-app-build@sha256:$docker_image_hash
}

function build_wireguard_go {
    if is_android_build $@; then
        build_android
        return
    fi

    local platform="$(uname -s)";
    case  "$platform" in
        Linux*|Darwin*) build_unix $platform;;
        MINGW*|MSYS_NT*) build_windows;;
    esac
}

build_wireguard_go $@
