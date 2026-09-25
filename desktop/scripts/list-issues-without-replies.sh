#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

REPO_ROOT=../../

source $REPO_ROOT/scripts/utils/log

GH_ARGS=(--state open --limit 500)

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
        *)
            log_info "Unknown parameter: $1"
            echo ""
            print_usage
            exit 1
            ;;
    esac
    shift
done

gh issue list "${GH_ARGS[@]}" --json title,url,createdAt,comments,number \
    --jq 'map(select(any(.comments[]; .authorAssociation == "MEMBER") | not)) | sort_by(.createdAt) | reverse | .[] | "\(.createdAt[:10])\t\(.number)\t\(.title[:58])\t\(.url)"' | \
        while IFS=$'\t' read -r createdDate number title url; do
          printf '[%s] %s (\e]8;;%s\e\\%s\e]8;;\e\\)\n' "$createdDate" "$title" "$url" "#$number"
        done

