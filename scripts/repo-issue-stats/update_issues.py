#!/usr/bin/env python3
"""
Update local GitHub issues by fetching new issues from the API.
Only downloads issues that don't already exist locally.
"""

import requests
import json
import os
import sys
from pathlib import Path


def get_existing_issue_numbers(directory):
    """Get a set of all issue numbers already saved locally."""
    issue_numbers = set()

    if not os.path.exists(directory):
        return issue_numbers

    for filename in os.listdir(directory):
        if filename.endswith('.json'):
            try:
                issue_num = int(filename[:-5])  # Remove .json extension
                issue_numbers.add(issue_num)
            except ValueError:
                pass  # Skip files that don't match the pattern

    return issue_numbers


def fetch_new_issues(repo, issues_dir, token=None):
    """Fetch new issues from GitHub API and save them locally."""

    # Get existing issue numbers
    existing_issues = get_existing_issue_numbers(issues_dir)

    if existing_issues:
        max_existing = max(existing_issues)
        print(f"Found {len(existing_issues)} existing issues (highest: #{max_existing})")
    else:
        max_existing = 0
        print("No existing issues found")

    # Prepare API request
    api_url = f"https://api.github.com/repos/{repo}/issues"
    headers = {}
    if token:
        headers['Authorization'] = f'token {token}'
        print("Using GitHub token for authentication")
    else:
        print("No GitHub token provided (rate limits apply)")

    # Create directory if it doesn't exist
    os.makedirs(issues_dir, exist_ok=True)

    new_issues_count = 0
    page = 1

    while True:
        params = {
            'state': 'all',
            'per_page': 100,
            'sort': 'created',
            'direction': 'desc',
            'page': page
        }

        print(f"Fetching page {page}...")
        response = requests.get(api_url, params=params, headers=headers)
        response.raise_for_status()

        # Check rate limit
        remaining = response.headers.get('X-RateLimit-Remaining')
        if remaining:
            print(f"  Rate limit remaining: {remaining}")

        issues = response.json()

        if not issues:
            print("No more issues to fetch")
            break

        # Process each issue
        found_existing = False
        for issue in issues:
            issue_number = issue['number']

            # Check if we already have this issue
            if issue_number in existing_issues:
                print(f"  Found existing issue #{issue_number}, stopping fetch")
                found_existing = True
                break

            # Save new issue
            filename = os.path.join(issues_dir, f"{issue_number}.json")
            with open(filename, 'w') as f:
                json.dump(issue, f, indent=4)

            new_issues_count += 1
            print(f"  Saved issue #{issue_number}")

        # Stop if we found an existing issue
        if found_existing:
            break

        # Check if there are more pages
        link_header = response.headers.get('Link', '')
        if 'rel="next"' not in link_header:
            print("Reached last page")
            break

        page += 1

    return new_issues_count


def main():
    if len(sys.argv) != 3:
        print("Usage: update_issues.py <owner/repo> <issues_directory>")
        print("Example: update_issues.py mullvad/mullvadvpn-app mullvadvpn-app.issues/")
        print()
        print("Optional: Set GITHUB_TOKEN environment variable for higher rate limits")
        print("  export GITHUB_TOKEN=your_token_here")
        sys.exit(1)

    repo = sys.argv[1]
    issues_dir = sys.argv[2]
    token = os.environ.get('GITHUB_TOKEN')

    try:
        print(f"Updating issues for {repo}")
        print(f"Target directory: {issues_dir}")
        print()

        new_count = fetch_new_issues(repo, issues_dir, token)

        print()
        print(f"Done! Downloaded {new_count} new issues")

    except Exception as e:
        print(f"Error: {e}")
        raise


if __name__ == "__main__":
    main()
