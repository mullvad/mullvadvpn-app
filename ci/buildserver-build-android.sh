#!/usr/bin/env bash

# # Setup instructions before this script will work
#
# * Follow the instructions in ../README.md
# * Import and trust the GPG keys of everyone who the build server should trust code from
# * Ensure that the machine running this script is allowed to upload to releases.mullvad.net.

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
LAST_BUILT_DIR="$SCRIPT_DIR/last-built"
UPLOAD_DIR="$SCRIPT_DIR/upload"
ANDROID_CREDENTIALS_DIR="$SCRIPT_DIR/credentials-android"

BRANCHES_TO_BUILD=("origin/master")
TAG_PATTERN_TO_BUILD=("^android/")

upload() {
  for f in MullvadVPN-*.{apk,aab}; do
    sha256sum "$f" > "$f.sha256"
    mv "$f" "$f.sha256" "$UPLOAD_DIR/"
  done
}

build_ref() {
  ref=$1
  tag=${2:-""}

  current_hash="$(git rev-parse $ref^{commit})"
  if [ -f "$LAST_BUILT_DIR/$current_hash" ]; then
    # This commit has already been built
    return 0
  fi

  echo ""
  echo "[#] $ref: $current_hash, building new packages."

  if [[ $ref == "refs/tags/"* ]] && ! git verify-tag $ref; then
    echo "!!!"
    echo "[#] $ref is a tag, but it failed GPG verification!"
    echo "!!!"
    sleep 60
    return 0
  elif [[ $ref == "refs/remotes/"* ]] && ! git verify-commit $current_hash; then
    echo "!!!"
    echo "[#] $ref is a branch, but it failed GPG verification!"
    echo "!!!"
    sleep 60
    return 0
  fi

  # Clean our working dir and check out the code we want to build
  rm -r dist/ 2&>/dev/null || true
  git reset --hard
  git checkout $ref
  git submodule update
  git clean -df

  echo "Building Android app"
  ANDROID_CREDENTIALS_DIR=$ANDROID_CREDENTIALS_DIR ./building/containerized-build.sh android --app-bundle || return 0

  # If there is a tag for this commit then we append that to the produced artifacts
  # A version suffix should only be created if there is a tag for this commit and it is not a release build
  if [[ -n "$tag" ]]; then
      # Remove disallowed version characters from the tag
      version_suffix="+${tag//[^0-9a-z_-]/}"
      # Will only match paths that include *-dev-* which means release builds will not be included
      # Pipes all matching names and their new name to mv
      pushd dist
      for original_file in MullvadVPN-*-dev-*{.apk,.aab}; do
          new_file=$(echo $original_file | sed -nE "s/^(MullvadVPN-.*-dev-.*)(\.apk|\.aab)$/\1$version_suffix\2/p")
          mv $original_file $new_file
      done
      popd
  fi

  (cd dist/ && upload) || return 0

  touch "$LAST_BUILT_DIR/$current_hash"
  echo "Successfully finished Android build at $(date)"
}

cd "$BUILD_DIR"

while true; do
  # Delete all tags. So when fetching we only get the ones existing on the remote
  git tag | xargs git tag -d > /dev/null

  git fetch --prune --tags 2> /dev/null || continue
  tags=( $(git tag | grep "$TAG_PATTERN_TO_BUILD") )

  for tag in "${tags[@]}"; do
    build_ref "refs/tags/$tag" "$tag"
  done

  for branch in "${BRANCHES_TO_BUILD[@]}"; do
    build_ref "refs/remotes/$branch"
  done

  sleep 240
done
