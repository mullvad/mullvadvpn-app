#!/usr/bin/env bash

# This script is used to build wireguard-go libraries for all the platforms.
#
# If "DAITA" support should be enabled, pass the `--daita` flag when invoking this script.

set -eu

# If Wireguard-go should be built with DAITA-support.
DAITA="false"
# If the target OS is Adnroid.
ANDROID="false"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --android) ANDROID="true";;
        --daita)   DAITA="true";;
        *)
            log_error "Unknown parameter: $1"
            exit 1
            ;;
    esac
    shift
done

function unix_target_triple {
    local platform
    platform="$(uname -s)"
    if [[ ("${platform}" == "Linux") ]]; then
        local arch
        arch="$(uname -m)"
        echo "${arch}-unknown-linux-gnu"
    elif [[ ("${platform}" == "Darwin") ]]; then
        local arch
        arch="$(uname -m)"
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
    # TODO: consider using `log_header` here
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

            CC="$(xcrun -sdk "$SDKROOT" --find clang) -arch $arch -isysroot $SDKROOT"
            export CC
            export CFLAGS="-isysroot $SDKROOT -arch $arch -I$SDKROOT/usr/include"
            export LD_LIBRARY_PATH="$SDKROOT/usr/lib"
            export CGO_CFLAGS="-isysroot $SDKROOT -arch $arch"
            export CGO_LDFLAGS="-isysroot $SDKROOT -arch $arch"
        fi
    fi


    # Build wiregaurd-go as a library
    pushd libwg
    if [[ "$DAITA" == "true" ]]; then
        pushd wireguard-go
        make libmaybenot.a LIBDEST="$OUT_DIR"
        popd
        go build -v --tags daita -o "$OUT_DIR"/libwg.a -buildmode c-archive
    else
        go build -v -o "$OUT_DIR"/libwg.a -buildmode c-archive
    fi
    popd
}

function build_android {
    echo "Building wireguard-go for android"

    ./libwg/build-android.sh
}

function build_wireguard_go {
    if [[ "$ANDROID" == "true" ]]; then
        build_android "$@"
        return
    fi

    local platform
    platform="$(uname -s)";
    case "$platform" in
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
build_wireguard_go "$@"
