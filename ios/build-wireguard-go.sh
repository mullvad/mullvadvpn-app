#!/usr/bin/env bash

# build-wireguard-go.sh
# A helper build script for WireGuardGoBridge via ExternalBuildSystem target in Xcode.
#
# ExternalBuildSystem target configuration:
# Build Tool: /bin/sh
# Arguments: build-wireguard-go.sh $(ACTION)
# Directory: $(SOURCE_ROOT)
# Pass build settings in environment: YES

# Passed by Xcode
ACTION=$1

if [ "$SOURCE_PACKAGES_PATH" == "" ]; then
  # When archiving, Xcode sets the action to "install"
  if [ "$ACTION" == "install" ]; then
    SOURCE_PACKAGES_PATH="$BUILD_DIR/../../../../../SourcePackages"
  else
    SOURCE_PACKAGES_PATH="$BUILD_DIR/../../SourcePackages"
  fi
fi

# Resolve SourcesPackages path
RESOLVED_SOURCE_PACKAGES_PATH="$( cd "$SOURCE_PACKAGES_PATH" && pwd -P )"
if [ "$RESOLVED_SOURCE_PACKAGES_PATH" == "" ]; then
  echo "Failed to resolve the SourcePackages path: $SOURCE_PACKAGES_PATH"
  exit -1
fi

# Compile the path to the Makefile directory
WIREGUARD_KIT_GO_PATH="$RESOLVED_SOURCE_PACKAGES_PATH/checkouts/wireguard-apple/Sources/WireGuardKitGo"
echo "WireGuardKitGo path resolved to $WIREGUARD_KIT_GO_PATH"

# Run make
/usr/bin/make -C "$WIREGUARD_KIT_GO_PATH" $ACTION