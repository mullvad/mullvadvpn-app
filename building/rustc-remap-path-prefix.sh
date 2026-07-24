#!/usr/bin/env bash

# Returns a cargo argument that makes rustc replace the machine specific file
# paths it would otherwise record in the build artifacts with fixed values.
# Required for reproducible builds, where the same source has to build into
# the same bytes no matter who builds it or where.
#
# A `--config` argument rather than RUSTFLAGS, because RUSTFLAGS replaces the
# rustflags from `.cargo/config.toml` while `--config` is merged with them.
#
# Setting MULLVAD_DISABLE_PATH_REMAPPING to anything but 0 returns an argument
# that remaps nothing. Remapped paths keep debuggers and backtraces from
# finding the sources.

set -eu

# Emits a cargo `--config` argument setting rustflags to its arguments.
#
# `cfg(all())` matches every target, and cargo joins the rustflags of all
# matching target sections, so one argument covers them all and adds to what
# `.cargo/config.toml` sets rather than replacing it. No arguments is a no-op.
#
# cargo parses --config as TOML, so each flag becomes a TOML basic string, in
# which backslash and double quote have to be escaped.
function emit_config {
    local toml=""
    for flag in "$@"; do
        local escaped=${flag//\\/\\\\}     # backslashes, which Windows paths are full of
        escaped=${escaped//\"/\\\"}        # double quotes, which Unix paths may contain
        toml+="${toml:+,}\"$escaped\""
    done
    printf -- '--config=target."cfg(all())".rustflags=[%s]\n' "$toml"
}

if [[ ${MULLVAD_DISABLE_PATH_REMAPPING:-0} != "0" ]]; then
    emit_config
    exit 0
fi

# `pwd -P` because cargo resolves symlinks in the paths it records.
SOURCE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd .. && pwd -P )"

CARGO_HOME_PATH=${CARGO_HOME:-$HOME/.cargo}
RUSTUP_HOME_PATH=${RUSTUP_HOME:-$HOME/.rustup}
CARGO_TARGET_DIR_PATH=${CARGO_TARGET_DIR:-$SOURCE_DIR/target}

# cygpath converts Git-Bash style paths ("/c/Users/...") into native Windows
# paths ("C:\Users\..."). rustc matches the prefixes as plain text against the
# native Windows paths.
case "$(uname -s)" in
    MINGW*|MSYS_NT*)
        CARGO_HOME_PATH=$(cygpath -w "$CARGO_HOME_PATH")
        RUSTUP_HOME_PATH=$(cygpath -w "$RUSTUP_HOME_PATH")
        SOURCE_DIR=$(cygpath -w "$SOURCE_DIR")
        CARGO_TARGET_DIR_PATH=$(cygpath -w "$CARGO_TARGET_DIR_PATH")
        ;;
esac

# The order is significant. Iterated over in reverse, so last argument is
# treated as most significant. Since CARGO_TARGET_DIR_PATH might be located
# under $SOURCE_DIR, it's important to keep it last.
# See: https://github.com/rust-lang/rust/blob/55459598c250d985eb5f840306dfb59f267c03b6/compiler/rustc_span/src/source_map.rs#L1125-L1127
remap_flags=(
    "--remap-path-prefix=$CARGO_HOME_PATH=/CARGO_HOME"
    "--remap-path-prefix=$RUSTUP_HOME_PATH=/RUSTUP_HOME"
    "--remap-path-prefix=$SOURCE_DIR=/SOURCE_DIR"
    "--remap-path-prefix=$CARGO_TARGET_DIR_PATH=/CARGO_TARGET_DIR"
)

emit_config "${remap_flags[@]}"
