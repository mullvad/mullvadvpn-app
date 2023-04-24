#!/usr/bin/env bash

# CI/Developer script to format
# Relies on Tidy - https://github.com/htacg/tidy-html5

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

function main {
    case ${1:-""} in
        format) format;;
        formatAndCheckDiff) format && checkDiff;;
        "")
            echo "Available subcommands: format, formatAndCheckDiff"
            ;;
        *)
            echo "Unknown parameter: $1"
            exit 1
            ;;
    esac
}

# Autoformats Android XML files
function format {
    tidy -xml \
        -m  \
        -i  \
        -w 100 \
        -utf8 \
        --quiet yes \
        --indent-attributes yes \
        --indent-spaces 4 \
        --literal-attributes yes \
        ../app/src/main/AndroidManifest.xml \
        ../app/src/main/res/anim*/*.xml \
        ../app/src/main/res/drawable*/*.xml \
        ../app/src/main/res/layout*/*.xml \

    tidy -xml \
        -m  \
        -i  \
        -w 0 \
        -utf8 \
        --quiet yes \
        --indent-spaces 4 \
        --literal-attributes yes \
        --indent-cdata yes \
        ../app/src/main/res/values/*.xml        

    # FIXME - when tidy learns to not leave whitespace around, remove the line below - https://github.com/htacg/tidy-html5/issues/864
    find ../app/src/main/ -name '*.xml' -exec sed -i -e 's/[ \t]*$//' '{}' ';'
}

function checkDiff {
    if git diff --exit-code -- ../app/src/main/AndroidManifest.xml ../app/src/main/res; then
        echo "Android XML files are correctly formatted"
        return 0
    else
        echo "Android XML files are NOT correctly formatted"
        return 1
    fi
}

main "$@"
