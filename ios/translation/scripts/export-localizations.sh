#!/usr/bin/env bash
# export-localizations.sh
# Exports Swift/SwiftUI localization files (.xliff) from an Xcode project.

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/export-localization_$(date +%Y%m%d_%H%M%S).log"
TMP_LOG="$(mktemp)"

PROJECT_NAME="MullvadVPN"
SCHEME_NAME="$PROJECT_NAME"
XCODE_PROJECT_PATH="$SCRIPT_DIR/../../$PROJECT_NAME.xcodeproj"

EXPORT_LOCALIZATION_DIR="$SCRIPT_DIR/../locales"
TMP_EXPORT_DIR="${EXPORT_LOCALIZATION_DIR}/all_tmp_languages"

EXPORT_LANGUAGES=${EXPORT_LANGUAGES:-"en"}
CONFIGURATION="Debug"
BUILD_OUTPUT_DIR="$SCRIPT_DIR/build"
DERIVED_DATA_DIR="$BUILD_OUTPUT_DIR/derivedData"

trap 'on_fail' ERR

on_fail() {
  set +e
  echo "❌ Export failed. Cleaning up and saving log..."
  cleanup_build_folder
  cleanup_temp_folder
  mkdir -p "$(dirname "$LOG_FILE")"
  cat "$TMP_LOG" >"$LOG_FILE"
  echo "💥 Full log saved to: $LOG_FILE"
  exit 1
}

cleanup_build_folder() {
  echo "🧹 Cleaning build folder at: $BUILD_OUTPUT_DIR"
  rm -rf "$BUILD_OUTPUT_DIR"
}

cleanup_temp_folder() {
  echo "🧹 Cleaning temp folder at: $TMP_EXPORT_DIR"
  rm -rf "$TMP_EXPORT_DIR"
}

exec > >(tee "$TMP_LOG") 2>&1

build_project() {
  echo "👉 Building project..."
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
}

export_localizations() {
  echo "🌍 Exporting localizations for languages: $EXPORT_LANGUAGES"

  IFS=',' read -r -a LANG_ARRAY <<<"$EXPORT_LANGUAGES"

  for lang in "${LANG_ARRAY[@]}"; do
    echo "➡️ Exporting $lang"
    xcodebuild -exportLocalizations \
      -project "$XCODE_PROJECT_PATH" \
      -scheme "$SCHEME_NAME" \
      -derivedDataPath "$DERIVED_DATA_DIR" \
      -localizationPath "$TMP_EXPORT_DIR" \
      -exportLanguage "$lang"

    local xcloc_dir="${TMP_EXPORT_DIR}/${lang}.xcloc"

    if [[ -d "$xcloc_dir" ]]; then
      local xliff_file
      xliff_file=$(find "$xcloc_dir" -name '*.xliff' | head -n 1)
      if [[ -f "$xliff_file" ]]; then
        cp "$xliff_file" "$EXPORT_LOCALIZATION_DIR/${lang}.xliff"
        echo "✔️ Extracted $lang.xliff for Crowdin upload"
      else
        echo "❌ No .xliff file found in $xcloc_dir"
        false
      fi
    else
      echo "❌ .xcloc bundle not found for $lang"
      false
    fi
  done
}

main() {
  echo "📝 Export script started at: $(date)"
  build_project
  export_localizations
  cleanup_build_folder
  cleanup_temp_folder
  echo "🎉 Export complete. Crowdin-ready .xliff files are in: $EXPORT_LOCALIZATION_DIR"
  echo "✅ Script finished at: $(date)"
  rm -f "$TMP_LOG"
}

main
