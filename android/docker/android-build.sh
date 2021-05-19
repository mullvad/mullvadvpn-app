#!/usr/bin/env bash

set -eu

cp build-apk.sh /tmp/build-apk.sh

if [ -d wireguard/wireguard-go ]; then
    WGGO_DIR="wireguard/wireguard-go"
elif [ -d wireguard/libwg ]; then
    WGGO_DIR="wireguard/libwg"
else
    WGGO_DIR="/tmp"
    echo '#!/bin/bash' > /tmp/build-android.sh
    echo 'echo "Skipping WireGuard-Go build"' >> /tmp/build-android.sh
    chmod +x /tmp/build-android.sh
fi

cp "$WGGO_DIR/build-android.sh" /tmp/build-android-wireguard.sh

if [ "$ARCHITECTURES" != "" ]; then
    GO_ARCHITECTURES=""

    for arch in $ARCHITECTURES; do
        case "$arch" in
            i686) GO_ARCHITECTURES="$GO_ARCHITECTURES x86"
                ;;
            armv7) GO_ARCHITECTURES="$GO_ARCHITECTURES arm"
                ;;
            aarch64) GO_ARCHITECTURES="$GO_ARCHITECTURES arm64"
                ;;
            *) GO_ARCHITECTURES="$GO_ARCHITECTURES $arch"
                ;;
        esac
    done

    sed -i -e "s|^ARCHITECTURES=.*$|ARCHITECTURES=\"$ARCHITECTURES\"|" ./build-apk.sh
    sed -i -e "s|^for arch in .*; do*$|for arch in $GO_ARCHITECTURES; do|" "$WGGO_DIR/build-android.sh"
fi

sed -i -e "s|^./wireguard/build-wireguard-go.sh --android$|./$WGGO_DIR/build-android.sh|" ./build-apk.sh
sed -i -e 's|^EXTRA_WGGO_ARGS=""$|EXTRA_WGGO_ARGS="--no-docker"|' ./build-apk.sh

./build-apk.sh

mv /tmp/build-apk.sh .
mv /tmp/build-android-wireguard.sh "./$WGGO_DIR/build-android.sh"
