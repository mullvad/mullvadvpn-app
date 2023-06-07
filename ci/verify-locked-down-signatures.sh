#!/usr/bin/env bash
set -eu
shopt -s nullglob

readonly SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
readonly REPO_DIR="$SCRIPT_DIR/.."

# In the CI environment we would like to import trusted public keys from a file,
# but not in our build environment
import_gpg_keys="false"

# The policy of enforcing lockfiles to be signed was not in place before this commit and
# as such some of the commits before are not signed
# The whitelisted commit can be set in order to allow github actions to only check changes
# since origin/main
whitelisted_commit="618130520"

while [ "$#" -gt 0 ]; do
    case "$1" in
        "--import-gpg-keys")
            import_gpg_keys="true"
            ;;
        "--whitelist")
            whitelisted_commit="$2"
            shift
            ;;
        *)
            echo "Unknown argument $1
The options are --import-gpg-keys and --whitelist"
            exit 1
            ;;
    esac
    shift
done

if [[ "$import_gpg_keys" == "true" ]]; then
    GNUPGHOME=$(mktemp -d)
    export GNUPGHOME
    for key in "$SCRIPT_DIR"/keys/*; do
        gpg --import "$key"
    done
fi

# Parse the locked down files from the github actions workflow file.
# We need to define them there since github has no way to trigger on filepaths specified in a file.
# We parse them from there in order to avoid duplicating the locked down files in multiple places.
#
# This regexp line is using a regexp to parse the github .yml file for the YAML list
# that follows the `paths` key.
# It uses `tr` in order to turn the multi-lined file into a single-line that sed can parse
# correctly. This is done by replacing all new-lines with a `;`
readonly SEPARATOR=';'
locked_down_paths=$(\
    < "$REPO_DIR/.github/workflows/verify-locked-down-signatures.yml" tr '\n' $SEPARATOR \
    | sed "s/.*paths:$SEPARATOR\(\(\s*-\s[a-zA-Z\/\.-]*$SEPARATOR\)*\).*/\1/" \
    | tr $SEPARATOR '\n' \
    | awk '{print $2}')



unsigned_commits_exist=0
important_file_was_removed=0
for locked_path in $locked_down_paths; do
    echo "Checking $locked_path"

    locked_path_commit_hashes=$(git rev-list --oneline "$whitelisted_commit"..HEAD \
        "$REPO_DIR/$locked_path" | awk '{print $1}')

    for commit in $locked_path_commit_hashes; do
        echo -e "\tfound a change in $commit.."
        if ! git verify-commit "$commit" 2> /dev/null; then
            echo "Commit $commit which changed $locked_path is not signed."
            unsigned_commits_exist=1
        fi
    done

    # Check if important file has been removed.
    if [[ ! -e "$REPO_DIR/$locked_path" ]]; then
        echo "$locked_path was removed. If this was intentional, remove it from `verify-locked-down-signatures.yml`."
        important_file_was_removed=1
    fi
done

if [[ "$unsigned_commits_exist" != 0 || "$important_file_was_removed" != 0 ]]; then
    exit 1
fi

echo "SUCCESS: Could not find any offenses to locked down paths"
