#!/usr/bin/env bash

# This script generates the PNG/ICO menubar icons from the SVG files in `/menubar-icons/svg/`.
# Please see /menubar-icons/README.md for more information.

set -eu

if ! command -v convert > /dev/null; then
    echo >&2 "convert (imagemagick) is required to run this script"
    exit 1
fi

if ! command -v rsvg-convert > /dev/null; then
    echo >&2 "rsvg-convert (librsvg) is required to run this script"
    exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

MENUBAR_ICONS_DIR="../assets/images/menubar-icons"

SVG_DIR="$MENUBAR_ICONS_DIR/svg"
MACOS_DIR="$MENUBAR_ICONS_DIR/darwin"
WINDOWS_DIR="$MENUBAR_ICONS_DIR/win32"
LINUX_DIR="$MENUBAR_ICONS_DIR/linux"
TMP_DIR=$(mktemp -d)

COMPRESSION_OPTIONS=(
    -define png:compression-filter=5
    -define png:compression-level=9
    -define png:compression-strategy=1
    -define png:exclude-chunk=all
    -strip
)

function main() {
    mkdir -p "$MACOS_DIR" "$WINDOWS_DIR" "$LINUX_DIR"

    for frame in {1..9}; do
        generate "lock-$frame" "lock-$frame"
    done
    # The monochrome source svg differs from the colored one. The red circle is a hole in the monochrome
    # one. "lock-10_mono.svg" is the same icon but with a hole instead of a circle.
    generate lock-10 lock-10_mono

    rmdir "$TMP_DIR"
}

# Genares the ico icons for the Windows tray icon. The ico consists of 3 different resolutions with
# 3 different bit depths each. Each icon is also available with and without notification dot.
function generate_ico() {
    local svg_source_path="$1"
    local ico_target_path="$2"

    local tmp_file_paths=()
    local notification_icon_tmp_file_paths=()
    for size in 16 32 48; do
        local padding=$((size / 16))
        local notification_icon_size=$((size / 2))
        local png_tmp_path="$TMP_DIR/$size"

        generate_square "$svg_source_path" "$png_tmp_path.png" \
            "${png_tmp_path}_notification.png" "$size" "$padding" "$notification_icon_size"

        # 4- and 8-bit versions for RDP
        convert -colors 256 +dither "$png_tmp_path.png" png8:"$png_tmp_path-8.png"
        convert -colors 16  +dither "$png_tmp_path-8.png" "$png_tmp_path-4.png"

        convert -colors 256 +dither "${png_tmp_path}_notification.png" \
            png8:"${png_tmp_path}_notification-8.png"
        convert -colors 16  +dither "${png_tmp_path}_notification-8.png" \
            "${png_tmp_path}_notification-4.png"

        tmp_file_paths+=("$png_tmp_path.png" "$png_tmp_path-8.png" "$png_tmp_path-4.png")
        notification_icon_tmp_file_paths+=(
            "${png_tmp_path}_notification.png"
            "${png_tmp_path}_notification-8.png"
            "${png_tmp_path}_notification-4.png"
        )
    done

    convert "${tmp_file_paths[@]}" "${COMPRESSION_OPTIONS[@]}" "$ico_target_path.ico"
    convert "${notification_icon_tmp_file_paths[@]}" "${COMPRESSION_OPTIONS[@]}" \
        "${ico_target_path}_notification.ico"

    rm "${tmp_file_paths[@]}"
    rm "${notification_icon_tmp_file_paths[@]}"
}

# Generates pngs both for regular icon and icon with notification symbol next to the icon, ending
# up with a rectangular icon.
function generate_rectangle() {
    local svg_source_path="$1"
    local png_target_path="$2"
    local png_notification_target_path="$3"
    local target_size=$4
    local target_padding=$5
    local notification_width=$6
    local target_size_no_padding=$((target_size - target_padding * 2))
    local png_tmp_path="$TMP_DIR/tmp.png"

    generate_lock_png "$svg_source_path" "$png_target_path" "$target_size" "$target_padding"
    append_notification_icon "$png_target_path" "$png_notification_target_path" \
        "$notification_width"

    rm "$png_tmp_path"
}

# Generates pngs both for regular icon and icon with notification symbol, ending up with a square
# icon since the notification dot is overlapping the lock.
function generate_square() {
    local svg_source_path="$1"
    local png_target_path="$2"
    local png_notification_target_path="$3"
    local target_size=$4
    local target_padding=$5
    local notification_width=$6
    local target_size_no_padding=$((target_size - target_padding * 2))
    local png_tmp_path="$TMP_DIR/tmp.png"

    generate_lock_png "$svg_source_path" "$png_target_path" "$target_size" "$target_padding"
    overlay_notification_icon "$png_target_path" "$png_notification_target_path" \
        "$notification_width"

    rm "$png_tmp_path"
}

# Generates the lock png
function generate_lock_png() {
    local svg_source_path="$1"
    local png_target_path="$2"
    local target_size=$3
    local target_padding=$4
    local target_size_no_padding=$((target_size - target_padding * 2))
    local png_tmp_path="$TMP_DIR/tmp.png"

    rsvg-convert -o "$png_tmp_path" -w $target_size_no_padding -h $target_size_no_padding \
        "$svg_source_path"
    convert -background transparent "$png_tmp_path" -gravity center \
        -extent "${target_size}x$target_size" "${COMPRESSION_OPTIONS[@]}" "$png_target_path"
}

# Creates a copy of the icon at $source_path and appends the notification symbol to it
function append_notification_icon() {
    local source_path="$1"
    local target_path="$2"
    local width="$3"
    local padding="${4:-0}"
    local size=$((width + 2))
    local notification_icon_tmp_path="$TMP_DIR/notification.png"

    rsvg-convert -o "$notification_icon_tmp_path" -w $size -h $size \
        --left "$padding" --page-width $((size + padding)) --page-height $size \
        "$SVG_DIR/notification.svg"
    convert -strip -background transparent -colorspace sRGB -gravity center \
        +append "$source_path" "$notification_icon_tmp_path" "$target_path"

    rm "$notification_icon_tmp_path"
}

# Creates a copy of the icon at $source_path and puts the notification symbol on top of it in the
# bottom right corner.
function overlay_notification_icon() {
    local source_path="$1"
    local target_path="$2"
    local size="$3"
    local notification_icon_tmp_path="$TMP_DIR/notification.png"

    rsvg-convert -o "$notification_icon_tmp_path" -w "$size" -h "$size" "$SVG_DIR/notification.svg"
    convert -strip -background transparent -composite -colorspace sRGB -gravity SouthEast \
        "$source_path" "$notification_icon_tmp_path" "$target_path"

    rm "$notification_icon_tmp_path"
}

# Generates all icon versions for a specific frame.
function generate() {
    local icon_name="$1"
    local svg_source_path="$SVG_DIR/$icon_name.svg"
    local monochrome_svg_source_path="$SVG_DIR/$2.svg"

    local black_svg_source_path="$TMP_DIR/black.svg"
    local white_svg_source_path="$TMP_DIR/white.svg"

    local macos_target_base_path="$MACOS_DIR/$icon_name"
    local linux_target_base_path="$LINUX_DIR/$icon_name"
    local windows_target_base_path="$WINDOWS_DIR/$icon_name"

    sed -E 's/#[0-9a-fA-f]{6}/#000000/g' "$monochrome_svg_source_path" > "$black_svg_source_path"
    sed -E 's/#[0-9a-fA-f]{6}/#FFFFFF/g' "$monochrome_svg_source_path" > "$white_svg_source_path"

    # MacOS colored
    generate_rectangle "$svg_source_path" "$macos_target_base_path.png" \
        "${macos_target_base_path}_notification.png" 22 3 4
    generate_rectangle "$svg_source_path" "$macos_target_base_path@2x.png" \
        "${macos_target_base_path}_notification@2x.png" 44 6 8

    # MacOS monochrome
    generate_rectangle "$black_svg_source_path" "${macos_target_base_path}Template.png" \
        "${macos_target_base_path}_notificationTemplate.png" 22 3 4
    generate_rectangle "$black_svg_source_path" "${macos_target_base_path}Template@2x.png" \
        "${macos_target_base_path}_notificationTemplate@2x.png" 44 6 8

    # Linux colored
    generate_square "$svg_source_path" "$linux_target_base_path.png" \
        "${linux_target_base_path}_notification.png" 48 4 24

    # Linux white
    generate_square "$white_svg_source_path" "${linux_target_base_path}_white.png" \
        "${linux_target_base_path}_white_notification.png" 48 4 24

    # Windows colored
    generate_ico "$svg_source_path" "$windows_target_base_path"

    # Windows monochrome
    generate_ico "$white_svg_source_path" "${windows_target_base_path}_white"
    generate_ico "$black_svg_source_path" "${windows_target_base_path}_black"

    rm "$black_svg_source_path" "$white_svg_source_path"
}

main

