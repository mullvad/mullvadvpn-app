#!/usr/bin/env bash

set -eu

echo "Waiting for pcscd..."
service pcscd start
timeout 5 sh -c 'until [ -e /run/pcscd/pcscd.comm ]; do sleep 0.1; done'

exec "$@"
