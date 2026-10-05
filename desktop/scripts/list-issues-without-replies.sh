#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

REPO_ROOT=../../

# shellcheck source-path=desktop/scripts
source ./repositories-env.sh
source $REPO_ROOT/scripts/utils/log

GH_ARGS=(--state open --limit 500)
FORMAT_FN="format_href"

function print_usage {
    log_info "Usage: $0 [--help|-h] [--since <date parsed by \`date -d\`>]"
}

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --help|-h)
            print_usage
            exit 1
            ;;
        --since)
            GH_ARGS+=(--search "created:>=$(date -d "$2" +%F)")
            shift
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
    printf '[%s] %s (#%s)\n' "$1" "$3" "$2"
}

function format_href {
    printf '[%s] %s (\e]8;;%s\e\\#%s\e]8;;\e\\)\n' "$1" "$3" "$4" "$2"
}

function format_markdown {
    printf '[%s] %s ([#%s](%s))\n' "$1" "$3" "$2" "$4"
}

for repo in "${REPOSITORIES[@]}"; do
    if ! issues=$(gh issue list --repo "mullvad/$repo" "${GH_ARGS[@]}" \
        --json title,url,createdAt,comments,number \
        --jq 'map(select(any(.comments[]; .authorAssociation == "MEMBER") | not)) | sort_by(.createdAt) | reverse' 2> /dev/null); then
        continue
    fi

    if [[ "$issues" == "[]" ]]; then
        continue
    fi

    echo "# $repo"
    echo "$issues" | jq -r '.[] | "\(.createdAt[:10])\t\(.number)\t\(.title[:58])\t\(.url)"' | \
        while IFS=$'\t' read -r createdDate number title url; do
          "$FORMAT_FN" "$createdDate" "$number" "$title" "$url"
        done
    echo ""
done
