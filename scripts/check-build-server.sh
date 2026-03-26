#!/usr/bin/env bash
#
# Check if the build server is producing builds for recent commits.
# Compares builds available on releases.mullvad.net against the main branch.
#
# Exit codes:
#   0 - Latest commit has all builds (healthy)
#   1 - Latest commit missing builds (failing)
#   2 - Script or network error

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RELEASES_URL="https://releases.mullvad.net/desktop/builds"
CURL_TIMEOUT=30

# Default options
COMMITS_TO_CHECK=10
GRACE_PERIOD_MINS=30
QUIET=false

# Desktop build targets
TARGETS=(
    ".exe"
    "_x64.exe"
    "_arm64.exe"
    ".pkg"
    "_amd64.deb"
    "_arm64.deb"
    "_x86_64.rpm"
    "_aarch64.rpm"
)

# JSON output array
JSON_RESULTS=()

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Check if the build server is producing builds for recent main branch commits.

Options:
    -n, --commits N          Number of commits to check (default: 10)
    -g, --grace-period MINS  Grace period in minutes (default: 30)
    -q, --quiet              Only output JSON, no stderr messages
    -h, --help               Show this help message

Exit codes:
    0 - Latest commit has all builds (healthy)
    1 - Latest commit missing builds (failing)
    2 - Script or network error
EOF
}

log() {
    if [[ "$QUIET" == false ]]; then
        echo "$@" >&2
    fi
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -n|--commits)
                COMMITS_TO_CHECK="$2"
                shift 2
                ;;
            -g|--grace-period)
                GRACE_PERIOD_MINS="$2"
                shift 2
                ;;
            -q|--quiet)
                QUIET=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log "Error: Unknown option $1"
                usage
                exit 2
                ;;
        esac
    done
}

get_commits() {
    cd "$REPO_ROOT"
    # Return full hashes - we'll truncate later when needed
    git log --format="%H %ct" main --max-count="$COMMITS_TO_CHECK"
}

get_commit_version() {
    local commit_hash="$1"
    local version
    if ! version=$(git show "${commit_hash}:dist-assets/desktop-product-version.txt" 2>/dev/null); then
        log "Error: Version file not found for commit ${commit_hash}"
        exit 2
    fi
    echo "$version" | tr -d '[:space:]'
}

check_build_exists() {
    local version="$1"
    local target="$2"
    local url="$RELEASES_URL/$version/MullvadVPN-${version}${target}"

    # Use curl with HEAD request to check if file exists
    # -f flag makes curl return exit code 22 for 404 errors
    # -I flag for HEAD request
    # -L flag to follow redirects
    if curl -f -I -L --max-time "$CURL_TIMEOUT" "$url" &>/dev/null; then
        return 0
    else
        return 1
    fi
}

get_build_listing() {
    local version="$1"
    local url="$RELEASES_URL/$version/"

    # Fetch the directory listing and extract available artifacts
    curl -s -L --max-time "$CURL_TIMEOUT" "$url" 2>/dev/null | \
        grep -oE 'MullvadVPN-[^"]+\.(exe|pkg|deb|rpm)' | \
        sort -u || true
}

hours_since_commit() {
    local commit_time="$1"
    local current_time
    current_time=$(date +%s)
    local diff=$((current_time - commit_time))
    echo "scale=1; $diff / 3600" | bc
}

main() {
    parse_args "$@"

    local current_time
    current_time=$(date +%s)
    local grace_period_seconds=$((GRACE_PERIOD_MINS * 60))

    log "Checking builds for last $COMMITS_TO_CHECK commits on main branch"
    log "Grace period: $GRACE_PERIOD_MINS minutes"
    log ""

    local commits
    commits=$(get_commits)

    if [[ -z "$commits" ]]; then
        log "Error: No commits found"
        exit 2
    fi

    local latest_commit=""
    local latest_has_builds=true
    local too_recent_count=0
    local with_builds_count=0
    local missing_builds_count=0
    local first_iteration=true

    # Read commits into array (newest first)
    local commit_array=()
    while IFS= read -r line; do
        commit_array+=("$line")
    done <<< "$commits"

    for commit_info in "${commit_array[@]}"; do
        local full_commit_hash
        local commit_time
        full_commit_hash=$(echo "$commit_info" | cut -d' ' -f1)
        commit_time=$(echo "$commit_info" | cut -d' ' -f2)

        # Get base version for this specific commit
        local base_version
        base_version=$(get_commit_version "$full_commit_hash")

        # Use short hash (6 chars) for version string to match releases.mullvad.net format
        local short_hash="${full_commit_hash:0:6}"
        local version="${base_version}-dev-${short_hash}"
        local age_hours
        age_hours=$(hours_since_commit "$commit_time")

        # Use short hash for display and tracking
        local commit_hash="$short_hash"

        # Track latest commit
        if [[ "$first_iteration" == true ]]; then
            latest_commit="$commit_hash"
            first_iteration=false
        fi

        log "Checking $commit_hash (age: ${age_hours}h, version: $version)"

        # Check if commit is too recent
        local time_diff=$((current_time - commit_time))
        if [[ $time_diff -lt $grace_period_seconds ]]; then
            log "  → Too recent (within grace period)"
            too_recent_count=$((too_recent_count + 1))
            JSON_RESULTS+=("{\"commit\":\"$commit_hash\",\"base_version\":\"$base_version\",\"version\":\"$version\",\"age_hours\":$age_hours,\"status\":\"too_recent\",\"has_builds\":null,\"missing_targets\":[]}")
            continue
        fi

        # Check for builds
        log "  Fetching build listing..."
        local build_listing
        build_listing=$(get_build_listing "$version")

        if [[ -z "$build_listing" ]]; then
            log "  → No builds directory found"
            missing_builds_count=$((missing_builds_count + 1))
            local missing_json=$(printf ',"%s"' "${TARGETS[@]}")
            missing_json="[${missing_json:1}]"
            JSON_RESULTS+=("{\"commit\":\"$commit_hash\",\"base_version\":\"$base_version\",\"version\":\"$version\",\"age_hours\":$age_hours,\"status\":\"missing\",\"has_builds\":false,\"missing_targets\":$missing_json}")

            # Track if latest commit is missing builds
            if [[ "$commit_hash" == "$latest_commit" ]]; then
                latest_has_builds=false
            fi
            continue
        fi

        # Check each target
        local missing_targets=()
        for target in "${TARGETS[@]}"; do
            local expected_file="MullvadVPN-${version}${target}"
            if ! echo "$build_listing" | grep -q "^${expected_file}$"; then
                missing_targets+=("$target")
            fi
        done

        if [[ ${#missing_targets[@]} -eq 0 ]]; then
            log "  ✓ All builds present"
            with_builds_count=$((with_builds_count + 1))
            JSON_RESULTS+=("{\"commit\":\"$commit_hash\",\"base_version\":\"$base_version\",\"version\":\"$version\",\"age_hours\":$age_hours,\"status\":\"ok\",\"has_builds\":true,\"missing_targets\":[]}")
        else
            log "  ✗ Missing: ${missing_targets[*]}"
            missing_builds_count=$((missing_builds_count + 1))
            local missing_json=$(printf ',"%s"' "${missing_targets[@]}")
            missing_json="[${missing_json:1}]"
            JSON_RESULTS+=("{\"commit\":\"$commit_hash\",\"base_version\":\"$base_version\",\"version\":\"$version\",\"age_hours\":$age_hours,\"status\":\"missing\",\"has_builds\":false,\"missing_targets\":$missing_json}")

            # Track if latest commit is missing builds
            if [[ "$commit_hash" == "$latest_commit" ]]; then
                latest_has_builds=false
            fi
        fi
    done

    # Determine overall status
    local status
    if [[ "$latest_has_builds" == true ]]; then
        status="healthy"
    else
        status="failing"
    fi

    # Build JSON output
    local results_json=""
    for ((i=0; i<${#JSON_RESULTS[@]}; i++)); do
        if [[ $i -gt 0 ]]; then
            results_json+=","
        fi
        results_json+="${JSON_RESULTS[$i]}"
    done

    cat <<EOF
{
  "status": "$status",
  "latest_commit": "$latest_commit",
  "latest_has_builds": $latest_has_builds,
  "grace_period_mins": $GRACE_PERIOD_MINS,
  "commits_checked": ${#commit_array[@]},
  "results": [$results_json],
  "summary": {
    "too_recent": $too_recent_count,
    "with_builds": $with_builds_count,
    "missing_builds": $missing_builds_count,
    "total_checked": ${#commit_array[@]}
  }
}
EOF

    # Exit with appropriate code
    if [[ "$latest_has_builds" == true ]]; then
        exit 0
    else
        exit 1
    fi
}

main "$@"
