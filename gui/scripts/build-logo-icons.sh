#!/usr/bin/env bash

# Icon guidlines for MacOS:
# https://developer.apple.com/design/human-interface-guidelines/macos/icons-and-images/app-icon/
#
# Icon guidlines for Windows:
# https://docs.microsoft.com/en-us/windows/uwp/design/style/app-icons-and-logos#target-size-app-icon-assets
# https://docs.microsoft.com/en-us/windows/win32/uxguide/vis-icons

set -eu

if ! command -v convert > /dev/null; then
    echo >&2 "convert (imagemagick) is required to run this script"
    exit 1
fi

if ! command -v rsvg-convert > /dev/null; then
    echo >&2 "rsvg-convert (librsvg) is required to run this script"
    exit 1
fi

if ! command -v iconutil > /dev/null; then
    echo >&2 "iconutil is required to run this script"
    exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

DIST_ASSETS_DIR="../../dist-assets"
SVG_SOURCE_PATH="$DIST_ASSETS_DIR/icon.svg"
TMP_DIR=$(mktemp -d)
TMP_ICONSET_DIR="$TMP_DIR/icon.iconset"

COMPRESSION_OPTIONS="-define png:compression-filter=5 -define png:compression-level=9 \
    -define png:compression-strategy=1 -define png:exclude-chunk=all -strip"

# MacOS and Linux .icns icon
mkdir $TMP_ICONSET_DIR
for size in 16 32 128 256 512; do
    double_size=$[$size * 2]
    rsvg-convert -o $TMP_ICONSET_DIR/icon-$size.png -w $size -h $size $SVG_SOURCE_PATH
    rsvg-convert -o $TMP_ICONSET_DIR/icon-$size@2x.png -w $double_size -h $double_size \
        $SVG_SOURCE_PATH
done
iconutil --convert icns --output  $DIST_ASSETS_DIR/icon.icns $TMP_ICONSET_DIR
rm -rf $TMP_ICONSET_DIR

# Windows .ico icon
for size in 16 20 24 30 32 36 40 48 60 64 72 80 96 256 512; do
    rsvg-convert -o $TMP_DIR/$size.png -w $size -h $size $SVG_SOURCE_PATH
done
convert $TMP_DIR/* $COMPRESSION_OPTIONS $DIST_ASSETS_DIR/icon.ico

rm -rf $TMP_DIR

