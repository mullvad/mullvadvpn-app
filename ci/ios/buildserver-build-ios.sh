#!/usr/bin/env bash
set -eu
shopt -s nullglob

TAG_PATTERN_TO_BUILD=("^ios/")
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app/ios"
LAST_BUILT_DIR="$SCRIPT_DIR/last-built"
mkdir -p "$LAST_BUILT_DIR"

# Convince git to work on our checkout of the app repository, regardless of PWD.
export GIT_WORK_TREE="$SCRIPT_DIR/mullvadvpn-app/"
export GIT_DIR="$GIT_WORK_TREE/.git"
function run_git {
    # `git submodule` needs more info than just $GIT_DIR and $GIT_WORK_TREE.
    # But -C makes it work.
    git -C "$GIT_WORK_TREE" "$@"
}


function build_ref() {
    local tag=$1;
    local current_hash="";
    if ! current_hash=$(run_git rev-parse "$tag^{commit}"); then
        echo "!!!"
        echo "[#] Failed to get commit for $tag"
        echo "!!!"
        return
    fi

    if [ -f "$LAST_BUILT_DIR/commit-$current_hash" ]; then
        # This commit has already been built
        return 0
    fi


    if ! run_git verify-tag "$tag"; then
        echo "!!!"
        echo "[#] $tag failed GPG verification!"
        echo "!!!"
        sleep 60
        return 0
    fi

    run_git reset --hard
    run_git checkout "$tag"
    run_git submodule update
    run_git clean -df

    local app_build_version="";
    if ! app_build_version=$(read_app_version); then
        echo "!!!"
        echo "[#] Failed to read app build version for tag $tag ($current_hash)"
        echo "!!!"
        return 0
    fi


    if [ -f "$LAST_BUILT_DIR/build-$app_build_version" ]; then
        echo "!!!"
        echo "[#] App version $app_build_version already built in commit $(cat "${LAST_BUILT_DIR}/build-${app_build_version}")"
        echo "[#] The build version in Configuration/Version.xcconfig should be bumped."
        echo "[#] Failed to build $current_hash"
        echo "!!!"
        return 0
    fi

    echo ""
    echo "[#] $tag: $app_build_version $current_hash, building new packages."

    if "$SCRIPT_DIR"/run-build-and-upload.sh; then
        touch "$LAST_BUILT_DIR"/"commit-$current_hash"
        echo "$current_hash" > "$LAST_BUILT_DIR"/"build-${app_build_version}"
        echo "Successfully built ${app_build_version} ${tag} with hash ${current_hash}"
    fi
}

function read_app_version() {
    project_version=$(sed -n "s/CURRENT_PROJECT_VERSION = \([[:digit:]]\)/\1/p" "$BUILD_DIR/Configurations/Version.xcconfig")
    marketing_version=$(sed -n "s/MARKETING_VERSION = \([[:digit:]]\)/\1/p" "$BUILD_DIR/Configurations/Version.xcconfig")
    echo "${marketing_version}-${project_version}"
    if [ -z "$project_version" ] || [ -z "$marketing_version" ]; then
        exit 1;
    fi
}



function run_build_loop() {
    while true; do
        # Delete all tags. So when fetching we only get the ones existing on the remote
        run_git tag | xargs git tag -d > /dev/null

        run_git fetch --prune --tags 2> /dev/null || continue
        local tags=( $(run_git tag | grep "$TAG_PATTERN_TO_BUILD") )

        for tag in "${tags[@]}"; do
          build_ref "refs/tags/$tag"
        done

        sleep 240
    done
}

run_build_loop
