#!/usr/bin/env bash
# localizations.sh
# Exports strings from and Imports them to an Xcode project.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"

TMP_LOG="$(mktemp)"
PROJECT_NAME="MullvadVPN"
SCHEME_NAME="$PROJECT_NAME"
XCODE_PROJECT_PATH="$SCRIPT_DIR/../../$PROJECT_NAME.xcodeproj"

LOCALIZATION_DIR="$SCRIPT_DIR/../locales"
TMP_EXPORT_DIR="${LOCALIZATION_DIR}/all_tmp_languages"

EXPORT_LANGUAGES=${EXPORT_LANGUAGES:-"en"}
CONFIGURATION="Debug"
BUILD_OUTPUT_DIR="$SCRIPT_DIR/build"
DERIVED_DATA_DIR="$BUILD_OUTPUT_DIR/derivedData"

trap 'on_fail' ERR

on_fail() {
  set +e
  echo "Export failed. Cleaning up and saving log..."
  cleanup_build_folder
  cleanup_temp_folder
  mkdir -p "$(dirname "$LOG_FILE")"
  cat "$TMP_LOG" >"$LOG_FILE"
  echo "Full log saved to: $LOG_FILE"
  exit 1
}

cleanup_build_folder() {
  rm -rf "$BUILD_OUTPUT_DIR"
}

cleanup_temp_folder() {
  rm -rf "$TMP_EXPORT_DIR"
}

exec > >(tee "$TMP_LOG") 2>&1

build_project() {
  echo "Building project..."
  if ! xcodebuild \
    -project "$XCODE_PROJECT_PATH" \
    -scheme "$SCHEME_NAME" \
    -destination 'generic/platform=iOS' \
    -configuration "$CONFIGURATION" \
    -derivedDataPath "$DERIVED_DATA_DIR" \
    -quiet \
    CODE_SIGNING_REQUIRED=NO \
    CODE_SIGNING_ALLOWED=NO \
    clean build >"$TMP_LOG" 2>&1; then
    echo "Failed to build project"
    on_fail
  fi
  echo "Build succeeded"
}

export_localizations() {
  echo "Exporting localizations for languages: $EXPORT_LANGUAGES"

  IFS=',' read -r -a LANG_ARRAY <<<"$EXPORT_LANGUAGES"

  for lang in "${LANG_ARRAY[@]}"; do
    # Run xcodebuild and capture errors
    if ! xcodebuild -exportLocalizations \
      -project "$XCODE_PROJECT_PATH" \
      -scheme "$SCHEME_NAME" \
      -derivedDataPath "$DERIVED_DATA_DIR" \
      -localizationPath "$TMP_EXPORT_DIR" \
      -exportLanguage "$lang" \
      -quiet \
      CODE_SIGNING_REQUIRED=NO \
      CODE_SIGNING_ALLOWED=NO \
      >"$TMP_LOG" 2>&1; then
      echo "Failed to export localization for $lang"
      on_fail
    fi

    local xcloc_dir="${TMP_EXPORT_DIR}/${lang}.xcloc"

    if [[ -d "$xcloc_dir" ]]; then
      local xliff_file
      xliff_file=$(find "$xcloc_dir" -name '*.xliff' | head -n 1)
      if [[ -f "$xliff_file" ]]; then
        cp "$xliff_file" "$LOCALIZATION_DIR/${lang}.xliff"
        echo "Extracted $lang.xliff for Crowdin upload"
      else
        echo "No .xliff file found in $xcloc_dir"
        false
      fi
    else
      echo ".xcloc bundle not found for $lang"
      false
    fi
  done
}

clean_xliff_translations() {
  xliff_dir="$LOCALIZATION_DIR"
  if [[ ! -d "$xliff_dir" ]]; then
    echo "Directory not found: $xliff_dir"
    return 1
  fi

  # Dictionary of keys to ignore for translation
  declare -A UNNEEDED_KEYS=(
    ["CFBundleName"]=1
    ["CFBundleDisplayName"]=1
    ["NSHumanReadableCopyright"]=1
    # Add more keys here if needed
  )
  for xliff in "$xliff_dir"/*.xliff; do
    if [[ -f "$xliff" ]]; then
      for key in "${!UNNEEDED_KEYS[@]}"; do
        sed -i '' -E "/<trans-unit[^>]*id=\"$key\"[^>]*>/,/<\/trans-unit>/d" "$xliff"
      done
    else
      echo "File not found: $xliff, skipping"
    fi
  done

}

import_localizations() {
  # Directory where the .xliff files are stored
  XLIFF_DIR="$LOCALIZATION_DIR"
  # Loop through each .xliff file in the directory
  for xliff_file in "$XLIFF_DIR"/*.xliff; do
    # Skip if no files found
    [ -e "$xliff_file" ] || continue

    # Extract language code from filename, e.g., fr.xliff → fr
    language_code=$(basename "$xliff_file" .xliff)

    echo "Importing localization: $language_code from $xliff_file"

    # Run xcodebuild and check for errors
    if ! xcodebuild -importLocalizations \
      -project "$XCODE_PROJECT_PATH" \
      -scheme "$SCHEME_NAME" \
      -derivedDataPath "$DERIVED_DATA_DIR" \
      -localizationPath "$xliff_file" \
      -exportLanguage "$language_code" \
      -quiet \
      CODE_SIGNING_REQUIRED=NO \
      CODE_SIGNING_ALLOWED=NO \
      >"$TMP_LOG" 2>&1; then
      echo "Failed to import $xliff_file"
      on_fail
    fi
  done
  echo "All localizations imported successfully."
}

localization_to_export() {
  LOG_FILE="$LOG_DIR/export-localization_$(date +%Y%m%d_%H%M%S).log"
  build_project
  export_localizations
  clean_xliff_translations
  cleanup_build_folder
  cleanup_temp_folder
  echo "Export complete. Crowdin-ready .xliff files are in: $LOCALIZATION_DIR"
  rm -f "$TMP_LOG"
}

localization_to_import() {
  LOG_FILE="$LOG_DIR/import-localization_$(date +%Y%m%d_%H%M%S).log"
  build_project
  import_localizations
  cleanup_build_folder
  cleanup_temp_folder
  echo "Import complete. Localized .xliff files have been imported to code"
  rm -f "$TMP_LOG"
}

# Main entrypoint
main() {
  case "${1:-}" in
  export)
    localization_to_export
    ;;
  import)
    localization_to_import
    ;;
  "")
    echo "Available subcommands: export, import"
    ;;
  *)
    echo "Unknown parameter: $1"
    exit 1
    ;;
  esac
}

main "$@"
