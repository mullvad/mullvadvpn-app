#!/usr/bin/env bash

set -eu

if ! command -v rsvg-convert > /dev/null; then
    echo >&2 "rsvg-convert (librsvg) is required to run this script"
    exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

ICON_SVG_PATH="../../graphics/icon.svg"
# Icons used for notification and quick settings tile
BLACK_MONO_ICON_PATH="../../graphics/icon-shaved.svg"

# The following helper function converts an SVG image into a PNG image for a specific DPI
#
# Parameters:
#   1. Path to source SVG image
#   2. DPI config, a string with two parameters separated by a '-'
#      a. the DPI specification, as the suffix of the output directory (e.g., mdpi, xxhdpi)
#      b. the size of the generated PNG image
#   3. (optional) The destination image name, if not specified it is assumed to be the same as the
#      input image name, with any '-'s replaced with '_'s
#   4. (optional) Destination directory type, either 'drawable' (the default) or 'mipmap'
#
# Examples:
#
# The following will generate a 50 by 50 image in android/app/src/main/res/drawable-hdpi/my_image.png
#
#     convert_image /tmp/my-image.svg hdpi-50
#
# The following will generate a 50 by 50 image in android/app/src/main/res/drawable-mdpi/other_image.png
#
#     convert_image /tmp/my-other-image.svg mdpi-50 other_image
#
# The following will generate a 50 by 50 image in android/app/src/main/res/mipmap-xxhdpi/my_icon.png
#
#     convert_image /tmp/my-final-image.svg xxhdpi-50 my_icon mipmap
function convert_image() {
    if (( $# < 2 )); then
        echo "Too few arguments passed to 'convert_image'" >&2
        exit 1
    fi

    local source_image="$1"
    local dpi_config="$2"

    if (( $# >= 3 )); then
        local destination_image="$3"
    else
        local destination_image="$(basename "$source_image" .svg | sed -e 's/-/_/g')"
    fi

    if (( $# >= 4 )); then
        local destination_dir="$4"
    else
        local destination_dir="drawable"
    fi

    local dpi="$(echo "$dpi_config" | cut -f1 -d'-')"
    local size="$(echo "$dpi_config" | cut -f2 -d'-')"

    local dpi_dir="../app/src/main/res/${destination_dir}-${dpi}"

    echo "$source_image -> ($size x $size) ${dpi_dir}/${destination_image}.png"
    mkdir -p "$dpi_dir"
    rsvg-convert "$source_image" -w "$size" -h "$size" -o "${dpi_dir}/${destination_image}.png"
}

# Launcher icon
for dpi_size in "mdpi-48" "hdpi-72" "xhdpi-96" "xxhdpi-144" "xxxhdpi-192"; do
    convert_image "$ICON_SVG_PATH" "$dpi_size" "ic_launcher" "mipmap"
done

# Logo used in some GUI areas
for dpi_size in "mdpi-50" "hdpi-75" "xhdpi-100" "xxhdpi-150" "xxxhdpi-200"; do
    convert_image "$ICON_SVG_PATH" "$dpi_size" "logo_icon"
done

# Large logo used in the launch screen
for dpi_size in "mdpi-120" "hdpi-180" "xhdpi-240" "xxhdpi-360" "xxxhdpi-480"; do
    convert_image "$ICON_SVG_PATH" "$dpi_size" "launch_logo"
done

# The white icon is generated from the black one
white_mono_icon_path="$(mktemp)"

sed -e 's/\(\.st1{.*\);fill:#000000;/\1;fill:#FFFFFF;/' "$BLACK_MONO_ICON_PATH" > "$white_mono_icon_path"

for dpi_size in "mdpi-24" "hdpi-36" "xhdpi-48" "xxhdpi-72" "xxxhdpi-96"; do
    convert_image "$BLACK_MONO_ICON_PATH" "$dpi_size" "small_logo_black"
    convert_image "$white_mono_icon_path" "$dpi_size" "small_logo_white"
done

rm "$white_mono_icon_path"
