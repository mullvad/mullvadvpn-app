#!/usr/bin/env bash

# This script creates the macOS .icns from the icons in /graphics/macOS/ which need to be updated
# first if the source SVGs have been updated. More info about how to update them can be found in
# the readme.
#
# Icon guidlines for macOS:
# https://developer.apple.com/design/human-interface-guidelines/macos/icons-and-images/app-icon/
#
# Icon templates for macOS:
# https://developer.apple.com/design/resources/
#
# Icon guidlines for Windows:
# https://docs.microsoft.com/en-us/windows/uwp/design/style/app-icons-and-logos#target-size-app-icon-assets
# https://docs.microsoft.com/en-us/windows/win32/uxguide/vis-icons

echo "Press enter to continue if you've followed the instructions in graphics/README.md"
read -r

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

GRAPHICS_DIR="../../graphics"
DIST_ASSETS_DIR="../../dist-assets"
SVG_SOURCE_PATH="$GRAPHICS_DIR/icon.svg"
TMP_DIR=$(mktemp -d)
TMP_ICO_DIR="$TMP_DIR/ico"
TMP_ICONSET_DIR="$TMP_DIR/icon.iconset"

mkdir $TMP_ICONSET_DIR
mkdir $TMP_ICO_DIR

COMPRESSION_OPTIONS="-define png:compression-filter=5 -define png:compression-level=9 \
    -define png:compression-strategy=1 -define png:exclude-chunk=all -strip"

# macOS .icns icon
for icon in "$GRAPHICS_DIR/macOS"/*; do
    cp "$icon" "$TMP_ICONSET_DIR"/
done

iconutil --convert icns --output "$DIST_ASSETS_DIR/icon-macos.icns" "$TMP_ICONSET_DIR"
rm "$TMP_ICONSET_DIR"/*

# Linux .icns icon
for size in 16 32 128 256 512; do
    double_size=$[$size * 2]
    rsvg-convert -o $TMP_ICONSET_DIR/icon-$size.png -w $size -h $size $SVG_SOURCE_PATH
    rsvg-convert -o $TMP_ICONSET_DIR/icon-$size@2x.png -w $double_size -h $double_size \
        $SVG_SOURCE_PATH
done
iconutil --convert icns --output $DIST_ASSETS_DIR/icon.icns $TMP_ICONSET_DIR
rm -rf $TMP_ICONSET_DIR

# Windows .ico icon
for size in 16 20 24 30 32 36 40 48 60 64 72 80 96 256 512; do
    rsvg-convert -o $TMP_ICO_DIR/$size.png -w $size -h $size $SVG_SOURCE_PATH
done
convert $TMP_ICO_DIR/* $COMPRESSION_OPTIONS $DIST_ASSETS_DIR/icon.ico
rm -rf $TMP_ICO_DIR

# Windows installer sidebar
# "bmp3" specifies the Windows 3.x format which is required for the image to be displayed
sidebar_path="$TMP_DIR/sidebar.png"
sidebar_logo_size=234
rsvg-convert -o $sidebar_path -w $sidebar_logo_size -h $sidebar_logo_size $SVG_SOURCE_PATH
convert -background "#294D73" $sidebar_path \
    -gravity center -extent ${sidebar_logo_size}x314 \
    -gravity west -crop 164x314+10+0 bmp3:$DIST_ASSETS_DIR/windows/installersidebar.bmp
rm $sidebar_path

# GUI notification icon
rsvg-convert -o ../assets/images/icon-notification.png -w 128 -h 128 $SVG_SOURCE_PATH

# GUI in app icon
cp "$SVG_SOURCE_PATH" ../assets/images/logo-icon.svg

rmdir $TMP_DIR

