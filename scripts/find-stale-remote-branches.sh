#!/usr/bin/env bash
#
# Find and print all remote branches not commited to in over two months.
# Prints grouped by commiter email. Used to detect stale branches that
# could potentially be removed.

TWO_MONTHS=$((60 * 60 * 24 * 60))

two_months_ago=$(($(date +%s) - TWO_MONTHS))

all_branches=$(git branch --remote | grep -v HEAD)
old_branches=()
# Filter out branches touched recently
for branch in $all_branches; do
    branch_commit_timestamp=$(git show -s --format=%at "$branch")
    if [[ $branch_commit_timestamp -lt $two_months_ago ]]; then
        old_branches+=("$branch")
    else
        echo "Skipping too new branch $branch"
    fi
done

echo ""
echo "=== Stale branches? ==="
echo ""

authors=$(for branch in "${old_branches[@]}"; do git show -s --format="%ae" "$branch"; done | sort -u)

for author in $authors; do
    echo "# $author #"
    for branch in "${old_branches[@]}"; do
        if [[ $(git show -s --format="%ae" "$branch") == "$author" ]]; then
            echo "  $branch"
        fi
    done
    echo ""
done
