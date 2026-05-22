#!/usr/bin/env bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

export FDROID_PUBLISH_ROOT_DIR="$SCRIPT_DIR/publish/fdroid"
export FDROID_REPOS_DIR="$FDROID_PUBLISH_ROOT_DIR/repos"
export FDROID_INBOX_DIR="$FDROID_PUBLISH_ROOT_DIR/inbox"
# Must be manually deployed and configured; see fdroid/rclone.conf.template.
export FDROID_RCLONE_CONFIG_PATH="$SCRIPT_DIR/credentials-android/rclone.conf"
