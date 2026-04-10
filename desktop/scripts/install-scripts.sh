#!/usr/bin/env bash
set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DESKTOP_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/../.." && pwd )"

source "$REPO_DIR/scripts/utils/log"

function desktop_ci() {
    desktop_pre_install
    pushd "$DESKTOP_DIR"
    npm ci --no-audit --no-fund
    popd
    desktop_post_install
}

function desktop_install() {
    desktop_pre_install
    pushd "$DESKTOP_DIR"
    npm install
    popd
    desktop_post_install
}


function desktop_post_install() {
    # Setup electron after install
    pushd "$DESKTOP_DIR/node_modules/electron"
    npm run postinstall
    popd

    # Run postinstall in our own packages
    pushd "$DESKTOP_DIR"
    npm run postinstall --if-present --ws
    popd
}

function desktop_pre_install() {
    # Run preinstall in our own packages
    pushd "$DESKTOP_DIR"
    npm run preinstall --if-present --ws
    popd
}

case ${1-:""} in
    ci)
        desktop_ci
    ;;
    install)
        desktop_install
    ;;
    *)
        log_error "Invalid argument. Specify 'ci' or 'install' as the first argument."
        exit 1
esac

