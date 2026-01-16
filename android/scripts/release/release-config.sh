#!/usr/bin/env bash

# Configuration variables shared between the release scripts in this directory.

# Where the release scripts and programs store temporary data
export DATA_DIR="$HOME/.local/share/mullvad-release-android"

export WORK_DIR="$DATA_DIR/work/"
export PUBLISHED_DIR="$DATA_DIR/currently_published/"

# The user on the buildserver that builds and uploads artifacts to the cdn servers
export BUILDSERVER_BUILDUSER="build"
