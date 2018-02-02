#!/usr/bin/env bash

# This script is used to build, and sign a release artifact. See `README.md` for instructions on
# how to just build a development/testing version.

set -eu

if [[ "${1:-""}" != "--allow-dirty" ]]; then
    if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
        echo "Dirty working directory!"
        echo "You should only build releases in clean working directories in order to make it"
        echo "easier to reproduce the same build."
        echo ""
        echo "Use --allow-dirty to skip this check. Never do this for official releases."
        exit 1
    fi
fi

case "$(uname -s)" in
    Darwin*)    export MACOSX_DEPLOYMENT_TARGET="10.7";;
esac

binaries=(
    ./target/release/mullvad-daemon
    ./target/release/mullvad
    ./target/release/problem-report
)

# Remove binaries. To make sure it is rebuilt with the stable toolchain and the latest changes.
for binary in ${binaries[*]}; do
    echo "Removing $binary"
    rm -f $binary
done

echo "Compiling Rust backend in release mode..."
cargo +stable build --release

for binary in ${binaries[*]}; do
    echo "Stripping debugging symbols from $binary"
    strip $binary
done

echo "Updating relay list..."
./target/release/list-relays > dist-assets/relays.json

echo "Installing JavaScript dependencies..."
yarn install

echo "Packing final release artifact..."
case "$(uname -s)" in
    #Linux*)     yarn pack:linux;;
    Darwin*)    yarn pack:mac;;
esac

RELEASE_VERSION=`./target/release/mullvad-daemon --version | cut -f2 -d' '`
echo "**********************************"
echo ""
echo " The build finished successfully! "
echo " You have built:"
echo ""
echo " $RELEASE_VERSION"
echo ""
echo "**********************************"
