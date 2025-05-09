#!/usr/bin/env bash

# Configuration variables shared between the release scripts in this directory.

# Where to download app installers locally during the release process.
# This value is also hardcoded into the `mullvad-release` binary and
# has to be in sync with that value
export ARTIFACT_DIR="artifacts"
