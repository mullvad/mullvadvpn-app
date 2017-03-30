#! /usr/bin/env bash

# Will make sure you have rustfmt at the version in $VERSION, then format all the source code.
# Run with --only-format as the first argument to skip checking rustfmt version.

set -u

VERSION="0.7.1"
CMD="rustfmt"
INSTALL_CMD="cargo install --vers $VERSION --force $CMD"

function correct_rustfmt() {
    if ! which $CMD; then
        echo "$CMD is not installed" >&2
        return 1
    fi
    local installed_version=$($CMD --version | cut -d' ' -f1)
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

find . -iname "*.rs" -not -path "*/target/*" -print0 | xargs -0 -n1 rustfmt "$@"
