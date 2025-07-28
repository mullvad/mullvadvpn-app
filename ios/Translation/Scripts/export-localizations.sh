#!/usr/bin/env bash
# export-localizations.sh
# Exports Swift/SwiftUI localization files (.xliff) from an Xcode project.

# === Set script directory and log file path FIRST ===
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/logs/export-localization-build_$(date +%Y%m%d_%H%M%S).log"
TMP_LOG="$(mktemp)"

# === Project and localization config ===
PROJECT_NAME="MullvadVPN"
SCHEME_NAME="$PROJECT_NAME"
XCODE_PROJECT_PATH="$SCRIPT_DIR/../../$PROJECT_NAME.xcodeproj"
EXPORT_LOCALIZATION_DIR="$SCRIPT_DIR/../locales"
EXPORT_LANGUAGES=""
DEFAULT_LANGUAGES="en,da,de,es,fi,fr,it,ja,ko,my,nb,nl,pl,pt,ru,sv,th,tr,zh-Hans,zh-Hant" # Default languages list
LANGUAGES="${EXPORT_LANGUAGES:-$DEFAULT_LANGUAGES}"                                       # Use user-provided list or default
CONFIGURATION="Debug"
BUILD_OUTPUT_DIR="$SCRIPT_DIR/build"
DERIVED_DATA_DIR="$BUILD_OUTPUT_DIR/derivedData"
PRUNE_LOGS_DAYS="3" # Set to number of days (e.g., 7) to enable log pruning
TMP_EXPORT_DIR="${EXPORT_LOCALIZATION_DIR}/all_tmp_languages"

# === Fail fast on errors or undefined vars ===
set -euo pipefail

# === Handle logging only on failure ===
cleanup_build_folder() {
  echo "🧹 Cleaning up build folder at: $BUILD_OUTPUT_DIR"
  rm -rf "$BUILD_OUTPUT_DIR"
}
cleanup_temp_folder() {
  echo "🧹 Cleaning up temp folder at: $BUILD_OUTPUT_DIR"
  rm -rf "$TMP_EXPORT_DIR"
}
on_fail() {
  set +e
  echo "❌ Build failed. Cleaning up and saving log..."

  # Custom cleanup logic
  cleanup_build_folder
  cleanup_temp_folder

  # Save log
  mkdir -p "$(dirname "$LOG_FILE")"
  cat "$TMP_LOG" >"$LOG_FILE"
  echo "💥 Full log saved to: $LOG_FILE"
}
trap 'on_fail' ERR

# === Pipe all output through temporary buffer ===
exec > >(tee "$TMP_LOG") 2>&1

# === Begin export ===
echo "📝 Logging started at: $(date)"
echo "🚀 Starting localization export for project: $PROJECT_NAME"
echo "📁 Script directory: $SCRIPT_DIR"

echo "👉 Cleaning and building the project to generate .strings files..."
xcodebuild \
  -project "$XCODE_PROJECT_PATH" \
  -scheme "$SCHEME_NAME" \
  -destination 'generic/platform=iOS' \
  -configuration "$CONFIGURATION" \
  -derivedDataPath "$DERIVED_DATA_DIR" \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGNING_ALLOWED=NO \
  clean build
echo "✅ Build succeeded"

echo ""
echo "👉 Exporting localizations from the build..."

# Convert to array
IFS=',' read -r -a LANG_ARRAY <<<"$LANGUAGES"
echo "Languages to export: ${LANG_ARRAY[*]}"

echo "🌍 Exporting localizations..."
for lang in "${LANG_ARRAY[@]}"; do

  xcodebuild \
    -exportLocalizations \
    -project "$XCODE_PROJECT_PATH" \
    -scheme "$SCHEME_NAME" \
    -derivedDataPath "$DERIVED_DATA_DIR" \
    -localizationPath "$TMP_EXPORT_DIR" \
    -exportLanguage "$lang"
  # Map Chinese locales if needed
  case "$lang" in
    "zh-Hans") export_folder="zh-CN" ;;
    "zh-Hant") export_folder="zh-TW" ;;
    *) export_folder="$lang" ;;
  esac

  XLOC_DIR="${TMP_EXPORT_DIR}/${lang}.xcloc"
  DEST_DIR="${EXPORT_LOCALIZATION_DIR}/${export_folder}"

  if [ -d "$XLOC_DIR" ]; then
    mkdir -p "$DEST_DIR"

    # Find the .xliff file inside the .xcloc bundle
    XLIFF_FILE=$(find "$XLOC_DIR" -name '*.xliff' | head -n 1)

    if [ -f "$XLIFF_FILE" ]; then
      cp "$XLIFF_FILE" "${DEST_DIR}/ios-strings.xliff"
      echo "Copied $XLIFF_FILE to ${DEST_DIR}/ios-strings.xliff"
    else
      echo "❌ No .xliff file found inside $XLOC_DIR"
      false
    fi
  else
    echo "❌ .xcloc folder for $lang not found: $XLOC_DIR"
    false
  fi
done

echo ""
cleanup_build_folder
cleanup_temp_folder

echo "🎉 Done. Localizations are exported to: $EXPORT_LOCALIZATION_DIR"
echo "✅ Script finished at: $(date)"

# Remove temporary log since everything succeeded
rm -f "$TMP_LOG"

# === Remove logs older than 7 days ===
function prune_old_logs {
  if [[ -z "$PRUNE_LOGS_DAYS" ]]; then
    echo "🛑 Log pruning disabled. Set PRUNE_LOGS_DAYS to enable."
    return
  fi

  LOG_DIR="$(dirname "$LOG_FILE")"
  echo "🗑  Pruning log files older than $PRUNE_LOGS_DAYS days in: $LOG_DIR"
  find "$LOG_DIR" -type f -name '*.log' -mtime +"$PRUNE_LOGS_DAYS" -delete || true
}

# Prune old logs
prune_old_logs
