#! /usr/bin/env bash
set -e

## Everything printed to stdout will be run by travis. So only print
## stuff that needs to be set, e.g. environment variables.

brew update > /dev/stderr
brew install czmq > /dev/stderr
