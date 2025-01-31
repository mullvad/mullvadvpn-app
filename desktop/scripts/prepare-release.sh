#!/usr/bin/env bash

# This script prepares for a release. Run it with the release version as the first argument and it
# will update version numbers, commit and add a signed tag.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

REPO_ROOT=../../

source $REPO_ROOT/scripts/utils/log

for argument in "$@"; do
    case "$argument" in
        -*)
            log_error "Unknown option \"$argument\""
            exit 1
            ;;
        *)
            PRODUCT_VERSION="$argument"
            ;;
    esac
done

changes_path=../packages/mullvad-vpn/changes.txt
changelog_path=$REPO_ROOT/CHANGELOG.md
product_version_path=$REPO_ROOT/dist-assets/desktop-product-version.txt

function checks {
    if [[ -z ${PRODUCT_VERSION+x} ]]; then
        log_error "Please give the release version as an argument to this script."
        log_error "For example: '2018.1-beta3' for a beta release, or '2018.6' for a stable one."
        exit 1
    fi

    if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
        log_error "Dirty working directory! Will not accept that for an official release."
        exit 1
    fi

    if [[ $(grep "CHANGE THIS BEFORE A RELEASE" $changes_path) != "" ]]; then
        log_error "It looks like you did not update $changes_path"
        exit 1
    fi

    if [[ $(grep "^## \\[$PRODUCT_VERSION\\] - " $changelog_path) == "" ]]; then
        log_error "It looks like you did not add $PRODUCT_VERSION to the changelog?"
        log_error "Please make sure the changelog is up to date and correct before you proceed."
        exit 1
    fi
}

function update_product_version {
    echo "$PRODUCT_VERSION" > $product_version_path
    git commit -S -m "Update desktop app version to $PRODUCT_VERSION" \
        $product_version_path
}

function create_tag {
    echo "Tagging current git commit with release tag $PRODUCT_VERSION..."
    git tag -s "$PRODUCT_VERSION" -m "$PRODUCT_VERSION"
}

checks
update_product_version
create_tag

log_success "================================================="
log_success "| DONE preparing for a release!                 |"
log_success "|    Now push the tag created by this script    |"
log_success "|    after you have verified it is correct:     |"
log_success "|        $ git push origin $PRODUCT_VERSION"
log_success "================================================="
