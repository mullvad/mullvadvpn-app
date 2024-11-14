# shellcheck shell=bash
#
# Sourcing this file should set up the appropriate environment for Visual Studio using vcvarsall.bat
#
# Currently, this script runs vcvarsall.bat and exports the following (after appropriate
# conversions):
# * PATH
# * INCLUDE

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

source "$SCRIPT_DIR/scripts/utils/host"

case $HOST in
    x86_64-pc-windows-msvc) HOST_TARGET=x64;;
    aarch64-pc-windows-msvc) HOST_TARGET=arm64;;
    *)
        log_error "Unexpected architecture: $HOST"
        exit 1
        ;;
esac

# Target architecture. Use the host architecture if unspecified.
TARGET=${TARGET:-"$HOST_TARGET"}

# Path to vcvarsall. This assumes that VS 2022 Community is available
VCVARSPATH="C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Auxiliary\\Build\\vcvarsall.bat"

if [[ ! -f "$VCVARSPATH" ]]; then
    echo -e "vcvarsall.bat not found. Please update the path in the script (${BASH_SOURCE[0]})"
    exit 1
fi

VCVARSENV=$(MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' cmd.exe /c "$VCVARSPATH" $TARGET \>nul \& set)

declare -A vcenvmap

function populate_vcenvmap {
    while IFS='=' read -r key value; do
        vcenvmap[$key]=$value
    done <<< "$VCVARSENV"
}

function to_unix_path {
    # Converts a Windows-style PATH to a UNIX-style PATH
    # eg from "C:\1\2\3;C:\4\5\6" to "/c/1/2/3:/c/4/5/6"
    echo $1 | sed -e 's|\([a-zA-Z]\):|\/\1|g' -e 's|\\|/|g' -e 's|;|:|g'
}

populate_vcenvmap

export INCLUDE="${vcenvmap["INCLUDE"]}"
export PATH="$(to_unix_path "${vcenvmap["PATH"]}")"

echo "Initialized VS environment for $TARGET"
