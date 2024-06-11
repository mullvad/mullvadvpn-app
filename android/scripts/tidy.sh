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
    non_text_xml_paths=($(find .. -wholename */src/*.xml ! -name strings*.xml ! -name plurals.xml))
    for xml_path in ${non_text_xml_paths[*]}; do
        tidy -xml \
            -m  \
            -i  \
            -w 100 \
            -utf8 \
            --quiet yes \
            --indent-attributes yes \
            --indent-spaces 4 \
            --literal-attributes yes \
            $xml_path
    done

    # We only format non-translated files since we don't want
    # to introduce a mismatch between the xml files and source
    # (.po) files.
    non_translated_text_xml_paths=($(find .. -wholename */values/strings*.xml -o -wholename */values/plurals.xml))
    for xml_path in ${non_translated_text_xml_paths[*]}; do
        tidy -xml \
            -m  \
            -i  \
            -w 0 \
            -utf8 \
            --quiet yes \
            --indent-spaces 4 \
            --literal-attributes yes \
            --indent-cdata yes \
            $xml_path
    done

    # FIXME - when tidy learns to not leave whitespace around, remove the line below - https://github.com/htacg/tidy-html5/issues/864
    find .. -name '*.xml' -exec sed -i -e 's/[ \t]*$//' '{}' ';'
}

function checkDiff {
    if git diff --exit-code -- ../**/*.xml; then
        echo "Android XML files are correctly formatted"
        return 0
    else
        echo "Android XML files are NOT correctly formatted"
        return 1
    fi
}

main "$@"
