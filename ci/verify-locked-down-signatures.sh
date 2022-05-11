#!/usr/bin/env bash
set -eu
shopt -s nullglob
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Import the trusted keys that we verify with
GNUPGHOME=$(mktemp -d)
gpg --import --armor $SCRIPT_DIR/trusted_keys.pub

# The policy of enforcing lockfiles to be signed was not in place before this commit and as such some of the commits before are not signed
# The whitelisted commit can be set in order to allow github actions to only check since origin/master
WHITELIST_COMMIT=${1:-"origin/master"}

unsigned_commits_exist=0
LOCKED_DOWN_FILES=$(cat $SCRIPT_DIR/locked_down_files.txt)
for locked_file in $LOCKED_DOWN_FILES;
do
    locked_file_commit_hashes=$(git rev-list --oneline $WHITELIST_COMMIT..HEAD $SCRIPT_DIR/../$locked_file | awk '{print $1}')
    for commit in $locked_file_commit_hashes;
    do
        if ! $(git verify-commit $commit 2> /dev/null); then
            echo Commit $commit changed $locked_file and is not signed.
            unsigned_commits_exist=1
        fi
    done
done

exit $unsigned_commits_exist
