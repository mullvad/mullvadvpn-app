#!/usr/bin/env bash

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

COMPRESSION_OPTIONS="-define png:compression-filter=5 -define png:compression-level=9 \
    -define png:compression-strategy=1 -define png:exclude-chunk=all -strip"

function generate_ico() {
    local svg_source_path="$1"
    local ico_target_path="$2"

    local tmp_file_paths=()
    for size in 16 32 48; do
        local png_tmp_path="$TMP_DIR/$size.png"
        local png8_tmp_path="$TMP_DIR/$size-8.png"
        local png4_tmp_path="$TMP_DIR/$size-4.png"

        rsvg-convert -o "$png_tmp_path" -w $size -h $size "$svg_source_path"
        convert -background transparent "$png_tmp_path" -gravity center -extent ${size}x$size \
            "$png_tmp_path"
        # 4- and 8-bit versions for RDP
        convert -colors 256 +dither "$png_tmp_path" png8:"$png8_tmp_path"
        convert -colors 16  +dither "$png8_tmp_path" "$png4_tmp_path"

        tmp_file_paths+=("$png_tmp_path" "$png8_tmp_path" "$png4_tmp_path")
    done

    convert "${tmp_file_paths[@]}" $COMPRESSION_OPTIONS "$ico_target_path"
    rm "${tmp_file_paths[@]}"
}

function generate_png() {
    local svg_source_path="$1"
    local png_target_path="$2"
    local target_size=$3
    local target_padding=$4
    local target_size_no_padding=$[$target_size - $target_padding * 2]
    local png_tmp_path="$TMP_DIR/tmp.png"

    rsvg-convert -o "$png_tmp_path" -w $target_size_no_padding -h $target_size_no_padding \
        "$svg_source_path"
    convert -background transparent "$png_tmp_path" -gravity center \
        -extent ${target_size}x$target_size $COMPRESSION_OPTIONS "$png_target_path"
    rm "$png_tmp_path"
}

function generate() {
    local icon_name="$1"
    local svg_source_path="$SVG_DIR/$icon_name.svg"
    local monochrome_svg_source_path="$SVG_DIR/$2.svg"

    local black_svg_source_path="$TMP_DIR/black.svg"
    local white_svg_source_path="$TMP_DIR/white.svg"

    sed -E 's/#[0-9a-fA-f]{6}/#000000/g' "$monochrome_svg_source_path" > "$black_svg_source_path"
    sed -E 's/#[0-9a-fA-f]{6}/#FFFFFF/g' "$monochrome_svg_source_path" > "$white_svg_source_path"

    # MacOS colored
    generate_png "$svg_source_path" "$MACOS_DIR/$icon_name.png" 22 3
    generate_png "$svg_source_path" "$MACOS_DIR/$icon_name@2x.png" 44 6

    # MacOS monochrome
    generate_png "$black_svg_source_path" "$MACOS_DIR/${icon_name}Template.png" 22 3
    generate_png "$black_svg_source_path" "$MACOS_DIR/${icon_name}Template@2x.png" 44 6

    # Linux colored
    generate_png "$svg_source_path" "$LINUX_DIR/$icon_name.png" 48 8

    # Linux white
    generate_png "$white_svg_source_path" "$LINUX_DIR/${icon_name}_white.png" 48 8

    # Windows colored
    generate_ico "$svg_source_path" "$WINDOWS_DIR/$icon_name.ico"

    # Windows white
    generate_ico "$white_svg_source_path" "$WINDOWS_DIR/${icon_name}_white.ico"
    generate_ico "$black_svg_source_path" "$WINDOWS_DIR/${icon_name}_black.ico"

    rm "$black_svg_source_path" "$white_svg_source_path"
}

mkdir -p "$MACOS_DIR" "$WINDOWS_DIR" "$LINUX_DIR"

for frame in {1..9}; do
    generate lock-$frame lock-$frame
done
# The monochrome source svg differs from the colored one. The red circle is a hole in the monochrome
# one. "lock-10_mono.svg" is the same icon but with a hole instead of a circle.
generate lock-10 lock-10_mono

rmdir "$TMP_DIR"

