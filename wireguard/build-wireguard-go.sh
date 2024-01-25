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

function is_docker_build {
    for arg in "$@"
    do
        case "$arg" in
            "--no-docker")
                return 1
        esac
    done
    return 0
}

function unix_target_triple {
    local platform="$(uname -s)"
    if [[ ("${platform}" == "Linux") ]]; then
        local arch="$(uname -m)"
        echo "${arch}-unknown-linux-gnu"
    elif [[ ("${platform}" == "Darwin") ]]; then
        local arch="$(uname -m)"
        if [[ ("${arch}" == "arm64") ]]; then
            arch="aarch64"
        fi
        echo "${arch}-apple-darwin"
    else
        echo "Can't deduce target dir for $platform"
        return 1
    fi
}


function build_unix {
    echo "Building wireguard-go for $1"

    # Flags for cross compiling
    if [[ "$(unix_target_triple)" != "$1" ]]; then
        # Linux arm
        if [[ "$1" == "aarch64-unknown-linux-gnu" ]]; then
            export CGO_ENABLED=1
            export GOARCH=arm64
            export CC=aarch64-linux-gnu-gcc
        fi

        # Environment flags for cross compiling on macOS
        if [[ "$1" == *-apple-darwin ]]; then
            export CGO_ENABLED=1
            export GOOS=darwin

            local arch=x86_64
            export GOARCH=amd64
            if [[ "$1" == aarch64-* ]]; then
                arch=arm64
                export GOARCH=arm64
            fi

            export CC="$(xcrun -sdk "$SDKROOT" --find clang) -arch $arch -isysroot $SDKROOT"
            export CFLAGS="-isysroot $SDKROOT -arch $arch -I$SDKROOT/usr/include"
            export LD_LIBRARY_PATH="$SDKROOT/usr/lib"
            export CGO_CFLAGS="-isysroot $SDKROOT -arch $arch"
            export CGO_LDFLAGS="-isysroot $SDKROOT -arch $arch"
        fi
    fi

    pushd libwg
        target_triple_dir="../../build/lib/$1"

        mkdir -p "$target_triple_dir"
        go build -v -o "$target_triple_dir"/libwg.a -buildmode c-archive
    popd
}

function build_android {
    echo "Building for android"

    if is_docker_build $@; then
        ../building/container-run.sh android wireguard/libwg/build-android.sh
    else
        ./libwg/build-android.sh
    fi
}

function build_wireguard_go {
    if is_android_build $@; then
        build_android $@
        return
    fi

    local platform="$(uname -s)";
    case  "$platform" in
        Linux*|Darwin*) build_unix "${1:-$(unix_target_triple)}";;
        *)
            echo "Unsupported platform"
            return 1
            ;;
    esac
}

# Ensure we are in the correct directory for the execution of this script
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$script_dir"
build_wireguard_go $@
