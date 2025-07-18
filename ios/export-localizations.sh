#!/usr/bin/env bash

# export-localizations.sh
# Exports Swift/SwiftUI localization files (.xliff) from an Xcode project.

set -euo pipefail

# Resolve the directory of the script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# The Xcode project name (without .xcodeproj extension)
PROJECT_NAME="MullvadVPN"

# The scheme to export localizations from
SCHEME_NAME="$PROJECT_NAME"

# Path to the .xcodeproj file
XCODE_PROJECT_PATH="$SCRIPT_DIR/$PROJECT_NAME.xcodeproj"

# Output directory for exported localizations
EXPORT_LOCALIZATION_DIR="$SCRIPT_DIR/Locale/ExportedLocalizations"

# Language to export (you can change or loop this)
EXPORT_LANGUAGE="en"

# Create output directory if needed
mkdir -p "$EXPORT_LOCALIZATION_DIR"

echo "📤 Exporting localizations from project: $PROJECT_NAME"
echo "📁 Localization output path: $EXPORT_LOCALIZATION_DIR"

# Run xcodebuild to export localizations
xcodebuild \
  -exportLocalizations \
  -project "$XCODE_PROJECT_PATH" \
  -scheme "$SCHEME_NAME" \
  -localizationPath "$EXPORT_LOCALIZATION_DIR" \
  -exportLanguage "$EXPORT_LANGUAGE"

echo "✅ Localization export complete for language: $EXPORT_LANGUAGE"
