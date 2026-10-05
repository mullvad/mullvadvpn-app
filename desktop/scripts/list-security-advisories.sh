#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

REPO_ROOT=../../

# shellcheck source-path=desktop/scripts
source ./repositories-env.sh
source $REPO_ROOT/scripts/utils/log

STATES=("triage")
FORMAT_FN="format_href"

function print_usage {
    log_info "Usage: $0 [--help|-h] [--all|-a] [--plain|--markdown|--href]"
}

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --help|-h)
            print_usage
            exit 1
            ;;
        --all|-a)
            STATES=()
            ;;
        --plain)
            FORMAT_FN="format_plain"
            ;;
        --markdown)
            FORMAT_FN="format_markdown"
            ;;
        --href)
            FORMAT_FN="format_href"
            ;;
        *)
            log_info "Unknown parameter: $1"
            echo ""
            print_usage
            exit 1
            ;;
    esac
    shift
done

function format_plain {
    printf '%s [%s]: %s\n' "$1" "$2" "$4"
}

function format_href {
    printf '\e]8;;%s\e\\%s\e]8;;\e\\ [%s]: %s\n' "$3" "$1" "$2" "$4"
}

function format_markdown {
    printf '[%s](%s) [%s]: %s\n' "$1" "$3" "$2" "$4"
}

for repo in "${REPOSITORIES[@]}"; do
    advisories=$(gh api "repos/mullvad/$repo/security-advisories")

    if [[ ${#STATES[@]} != 0 ]]; then
        advisories=$(echo -n "$advisories" | jq --args 'map(select(.state | IN($ARGS.positional[])))' \
            "${STATES[@]}")
    fi

    if [[ "$advisories" == "[]" ]]; then
        continue
    fi

    echo "# $repo"
    echo "$advisories" | jq -r '.[] | "\(.ghsa_id)\t\(.state)\t\(.html_url)\t\(.summary)"' | \
        while IFS=$'\t' read -r id state url summary; do
            "$FORMAT_FN" "$id" "$state" "$url" "$summary"
        done
    echo ""
done
