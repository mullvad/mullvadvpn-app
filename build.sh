#!/usr/bin/env bash

set -e

export MACOSX_DEPLOYMENT_TARGET="10.7"

cargo +stable build --release
