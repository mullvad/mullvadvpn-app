#!/usr/bin/env bash
set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app/ios"
LAST_BUILT_DIR="$SCRIPT_DIR/ios-last-built"
mkdir -p "$LAST_BUILT_DIR"


build_ref() {
    local tag=$1;
    local current_hash="";
    if ! current_hash=$(git rev-parse "$tag"); then
        echo "Failed to get commit for $tag"
        return
    fi

    if verify_tag "$tag" "$current_hash"; then
        echo "Building $tag with hash $current_hash"
        bash "$BUILD_DIR"/build.sh --deploy
        touch "$LAST_BUILT_DIR/$current_hash"
    fi
}


verify_tag() {
    local tag="${1}"
    local current_hash="${2}"

    [[ "$tag" == "refs/tags/ios-*" ]] && \
        git verify-tag "$tag" && \
        [ -f "$LAST_BUILT_DIR/$current_hash" ]
}


build_loop() {
    cd "$BUILD_DIR"
    while true; do
      git tag | xargs git tag -d > /dev/null
      git fetch --prune --tags 2> /dev/null || continue
      local tags=( $(git tag) )

      for tag in "${tags[@]}"; do
        build_ref "refs/tags/$tag" "$tag"
      done

      sleep 240
    done
}

build_loop
