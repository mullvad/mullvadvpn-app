#!/usr/bin/env sh

if ! command -v convert > /dev/null; then
  echo >&2 "convert (imagemagick) is required to run this script"
  exit 1
fi

if ! command -v rsvg-convert > /dev/null; then
  echo >&2 "rsvg-convert (librsvg) is required to run this script"
  exit 1
fi

MENUBAR_PATH="assets/images/menubar icons"

SVG="$MENUBAR_PATH/svg"
MACOS="$MENUBAR_PATH/darwin"
WINDOWS="$MENUBAR_PATH/win32"
LINUX="$MENUBAR_PATH/linux"
TMP="$MENUBAR_PATH/tmp"

MAKE_BLACK='s/#[0-9a-fA-f]{6}/#000000/g'
MAKE_WHITE='s/#[0-9a-fA-f]{6}/#FFFFFF/g'

COMPRESSION_OPTIONS="-define png:compression-filter=5 -define png:compression-level=9 \
  -define png:compression-strategy=1 -define png:exclude-chunk=all -strip"

function generateICO() {
  _IN="$1"
  _OUT="$2"

  for SIZE in 16 32 48; do
    _TMP="$TMP/$SIZE"

    rsvg-convert -o "$_TMP.png" -w $SIZE -h $SIZE "$_IN"
    convert -background transparent "$_TMP.png" -gravity center -extent ${SIZE}x$SIZE "$_TMP.png"
    # 4- and 8-bit versions for RDP
    convert -colors 256 +dither "$_TMP.png" png8:"$_TMP-8.png"
    convert -colors 16  +dither "$_TMP-8.png" "$_TMP-4.png"
  done

  convert "$TMP/16.png" "$TMP/32.png" "$TMP/48.png" "$TMP/16-8.png" "$TMP/32-8.png" \
    "$TMP/48-8.png" "$TMP/16-4.png" "$TMP/32-4.png" "$TMP/48-4.png" $COMPRESSION_OPTIONS "$_OUT"
  rm "$TMP/16.png" "$TMP/32.png" "$TMP/48.png" "$TMP/16-8.png" "$TMP/32-8.png" \
    "$TMP/48-8.png" "$TMP/16-4.png" "$TMP/32-4.png" "$TMP/48-4.png"
}

function generatePNG() {
  _IN="$1"
  _OUT="$2"
  SIZE=$3
  PADDING=$4
  SIZE_WO_PADDING=$[$SIZE - $PADDING * 2]

  rsvg-convert -o "$TMP/tmp.png" -w $SIZE_WO_PADDING -h $SIZE_WO_PADDING "$_IN"
  convert -background transparent "$TMP/tmp.png" -gravity center -extent ${SIZE}x$SIZE \
    $COMPRESSION_OPTIONS "$_OUT"
  rm "$TMP/tmp.png"
}

function generate() {
  IN="$SVG/$1.svg"
  IN_MONO="$SVG/$2.svg"
  OUT="$1"

  BLACK_SVG="$TMP/black.svg"
  WHITE_SVG="$TMP/white.svg"

  sed -E $MAKE_BLACK "$IN_MONO" > "$BLACK_SVG"
  sed -E $MAKE_WHITE "$IN_MONO" > "$WHITE_SVG"

  # MacOS colored
  generatePNG "$IN" "$MACOS/$OUT.png" 22 3
  generatePNG "$IN" "$MACOS/$OUT@2x.png" 44 6

  # MacOS monochrome
  generatePNG "$BLACK_SVG" "$MACOS/${OUT}Template.png" 22 3
  generatePNG "$BLACK_SVG" "$MACOS/${OUT}Template@2x.png" 44 6

  # Linux colored
  generatePNG "$IN" "$LINUX/$OUT.png" 48 8

  # Linux white
  generatePNG "$WHITE_SVG" "$LINUX/${OUT}_white.png" 48 8

  # Windows colored
  generateICO "$IN" "$WINDOWS/$OUT.ico"

  # Windows white
  generateICO "$WHITE_SVG" "$WINDOWS/${OUT}_white.ico"

  rm "$BLACK_SVG" "$WHITE_SVG"
}

mkdir -p "$MACOS" "$WINDOWS" "$LINUX" "$TMP"

for i in {1..9}; do
  generate lock-$i lock-$i
done
generate lock-10 lock-10_2

rmdir "$TMP"

