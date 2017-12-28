#! /usr/bin/env bash

# Will make sure you have rustfmt at the version in $VERSION, then format all the source code.
# Run with --only-format as the first argument to skip checking rustfmt version.

set -u

VERSION="0.3.4"
CMD="rustfmt"
INSTALL_CMD="cargo install --vers $VERSION --force rustfmt-nightly"

case "$(uname -s)" in
    Linux*)     export LD_LIBRARY_PATH=$(rustc +nightly --print sysroot)/lib;;
    Darwin*)    export DYLD_LIBRARY_PATH=$(rustc +nightly --print sysroot)/lib;;
    *)          echo "Unsupported platform"; exit 1
esac

function correct_rustfmt() {
    if ! which $CMD; then
        echo "$CMD is not installed" >&2
        return 1
    fi
    local installed_version=$($CMD --version | cut -d'-' -f1)
    if [[ "$installed_version" != "$VERSION" ]]; then
        echo "Wrong version of $CMD installed. Expected $VERSION, got $installed_version" >&2
        return 1
    fi
    return 0
}

if [[ "${1:-""}" != "--only-format" ]]; then
    if ! correct_rustfmt; then
        echo "Installing $CMD $VERSION"
        $INSTALL_CMD
    fi
else
    shift
fi

echo "Formatting..."
cargo +nightly fmt -- "$@"
