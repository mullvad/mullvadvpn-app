#!/usr/bin/env bash
# export-localizations.sh
# Exports Swift/SwiftUI localization files (.xliff) from an Xcode project.

# === Set script directory and log file path FIRST ===
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/Logs/build_$(date +%Y%m%d_%H%M%S).log"
TMP_LOG="$(mktemp)"

# === Fail fast on errors or undefined vars ===
set -euo pipefail

# === Handle logging only on failure ===
on_fail() {
    echo "❌ Script failed. Saving log to: $LOG_FILE"
    mkdir -p "$(dirname "$LOG_FILE")"
    cat "$TMP_LOG" >"$LOG_FILE"
    echo "💥 Full log copied to: $LOG_FILE"
}
trap 'on_fail' ERR

# === Pipe all output through temporary buffer ===
exec > >(tee "$TMP_LOG") 2>&1

# === Project and localization config ===
PROJECT_NAME="MullvadVPN"
SCHEME_NAME="$PROJECT_NAME"
XCODE_PROJECT_PATH="$SCRIPT_DIR/../$PROJECT_NAME.xcodeproj"
EXPORT_LOCALIZATION_DIR="$SCRIPT_DIR/ExportedLocalizations"
EXPORT_LANGUAGE="en"
CONFIGURATION="Debug"
BUILD_OUTPUT_DIR="$SCRIPT_DIR/Build"
DERIVED_DATA_DIR="$BUILD_OUTPUT_DIR/DerivedData"
PRUNE_LOGS_DAYS="" # Set to number of days (e.g., 7) to enable log pruning

# === Begin export ===
echo "📝 Logging started at: $(date)"
echo "🚀 Starting localization export for project: $PROJECT_NAME"
echo "📁 Script directory: $SCRIPT_DIR"

function cleanup_build_folder {
    echo "🧹 Cleaning up build folder at: $BUILD_OUTPUT_DIR"
    rm -rf "$BUILD_OUTPUT_DIR"
}

echo "👉 Cleaning and building the project to generate .strings files..."
if xcodebuild \
    -project "$XCODE_PROJECT_PATH" \
    -scheme "$SCHEME_NAME" \
    -destination 'generic/platform=iOS' \
    -configuration "$CONFIGURATION" \
    -derivedDataPath "$DERIVED_DATA_DIR" \
    CODE_SIGNING_REQUIRED=NO \
    CODE_SIGNING_ALLOWED=NO \
    clean build; then
    echo "✅ Build succeeded"
else
    echo "❌ Build failed"
    cleanup_build_folder
    exit 1
fi

echo ""
echo "👉 Exporting localizations from the build..."
if xcodebuild \
    -exportLocalizations \
    -project "$XCODE_PROJECT_PATH" \
    -scheme "$SCHEME_NAME" \
    -localizationPath "$EXPORT_LOCALIZATION_DIR" \
    -exportLanguage "$EXPORT_LANGUAGE" \
    -derivedDataPath "$DERIVED_DATA_DIR" \
    CODE_SIGNING_REQUIRED=NO \
    CODE_SIGNING_ALLOWED=NO; then
    echo "✅ Localization export succeeded"
else
    echo "❌ Localization export failed"
    cleanup_build_folder
    exit 1
fi

echo ""
cleanup_build_folder

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
