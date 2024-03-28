#!/usr/bin/env bash

set -ex

COMMIT=$(git rev-parse --short=6 HEAD)
npm run build-test-executable
cp ../dist/mullvadvpn-app-e2e-tests "$HOME/.cache/mullvad-test/packages/app-e2e-tests-2024.1-beta2-dev-${COMMIT}_amd64-unknown-linux-gnu"
cp ../dist/mullvadvpn-app-e2e-tests "$HOME/.cache/mullvad-test/packages/app-e2e-tests-2024.1-beta2-dev-${COMMIT}_x86_64-unknown-linux-gnu"
