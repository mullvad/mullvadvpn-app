#!/usr/bin/env bash

export FDROID_INBOX_DIR="$SCRIPT_DIR/publish/fdroid/inbox"

declare -A FDROID_REPO_DIRS=(
    [dev]="$SCRIPT_DIR/publish/fdroid/dev"
)

export RCLONE_CONFIG_PATH="$SCRIPT_DIR/cdn77-fdroid-rclone.conf"
