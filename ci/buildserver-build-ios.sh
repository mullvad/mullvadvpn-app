#!/usr/bin/env bash
set -eu
shopt -s nullglob

TAG_PATTERN_TO_BUILD=("^ios/")
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app/ios"
LAST_BUILT_DIR="$SCRIPT_DIR/last-built"
mkdir -p "$LAST_BUILT_DIR"


build_ref() {
    local ref=$1;
    local current_hash="";
    if ! current_hash=$(git rev-parse "$ref"); then
        echo "!!!"
        echo "[#] Failed to get commit for $ref"
        echo "!!!"
        return
    fi

    local app_build_version="";
    if ! app_build_version=$(read_build_version); then
        echo "!!!"
        echo "[#] Failed to read app build version"
        echo "!!!"
        return 0
    fi

    if [ -f "$LAST_BUILT_DIR/commit-$current_hash" ]; then
        # This commit has already been built
        return 0
    fi

    if [ -f "$LAST_BUILT_DIR/build-$app_build_version" ]; then
        echo "!!!"
        echo "[#] App version already built in commit $(cat "${LAST_BUILT_DIR}/build-${app_build_version}")"
        echo "[#] The build version in ios/Configuration/Version.xcconfig should be bumped."
        echo "!!!"
        return 0
    fi

    echo ""
    echo "[#] $ref: $app_build_version $current_hash, building new packages."

    if ! git verify-tag "$ref"; then
        echo "!!!"
        echo "[#] $ref failed GPG verification!"
        echo "!!!"
        sleep 60
        return 0
    fi

    bash "$BUILD_DIR"/build.sh --deploy
    echo "Successfully built ${app_build_version} ${tag} with hash ${current_hash}"
}

read_app_version() {
    project_version=$(sed -n "s/CURRENT_PROJECT_VERSION = \([[:digit:]]\)/\1/p" ios/Configurations/Version.xcconfig)
    marketing_version=$(sed -n "s/MARKETING_VERSION = \([[:digit:]]\)/\1/p" ios/Configurations/Version.xcconfig)
    echo "${marketing_version}-${project_version}"
}



build_loop() {
    cd "$BUILD_DIR"
    while true; do
      # Delete all tags. So when fetching we only get the ones existing on the remote
      git tag | xargs git tag -d > /dev/null

      git fetch --prune --tags 2> /dev/null || continue
      local tags=( $(git tag | grep "$TAG_PATTERN_TO_BUILD") )

      for tag in "${tags[@]}"; do
        build_ref "refs/tags/$tag" "$tag"
      done

      sleep 240
    done
}

build_loop
