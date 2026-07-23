#!/usr/bin/env python3
"""Fail if a Cargo.lock change pulls in a too-new crates.io dependency version.

This is the CI enforcement of the supply-chain policy in .cargo/config.toml:
freshly published crates are held back for a cooldown window so that malicious or
broken releases have time to be caught and yanked before we depend on them.

If a base git ref argument is given, only versions that are *new relative to
that ref* are checked, so the CI gate fires exactly when a change upgrades or adds a
dependency, never on versions that were already committed. With no argument,
every crates.io version in the lockfiles is checked.

Publish timestamps come from the `pubtime` field of the crates.io sparse index,
which is the same field the nightly min-publish-age feature looks at.

Crates listed in ci/rust-min-publish-age-allowlist.txt are exempt from the check.

The cooldown window lives only in .cargo/config.toml (registry.global-min-publish-age).
This script reads it from there, so there is a single source of truth.

Hopefully cargo develop native functionality that can replace this script eventually.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import NamedTuple

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
CONFIG = REPO_ROOT / ".cargo" / "config.toml"
ALLOWLIST = SCRIPT_DIR / "rust-min-publish-age-allowlist.txt"
INDEX_BASE = "https://index.crates.io"
USER_AGENT = "mullvadvpn-app min-publish-age check (github.com/mullvad/mullvadvpn-app)"

# Seconds per unit, matching cargo's duration parsing (a month is 30 days).
UNIT_SECONDS = {
    "second": 1, "seconds": 1,
    "minute": 60, "minutes": 60,
    "hour": 3600, "hours": 3600,
    "day": 86400, "days": 86400,
    "week": 604800, "weeks": 604800,
    "month": 2592000, "months": 2592000,
}


class MinPublishAge(NamedTuple):
    duration: timedelta
    text: str  # the verbatim config value, e.g. "7 days", shown in messages


def load_min_publish_age() -> MinPublishAge:
    with CONFIG.open("rb") as f:
        raw = tomllib.load(f).get("registry", {}).get("global-min-publish-age")
    if raw is None:
        sys.exit(f"error: registry.global-min-publish-age is not set in {CONFIG}")
    raw = raw.strip()
    if raw == "0":
        return MinPublishAge(timedelta(0), raw)
    match = re.fullmatch(r"(\d+)\s+(\w+)", raw)
    if not match or match.group(2) not in UNIT_SECONDS:
        sys.exit(f"error: cannot parse global-min-publish-age = {raw!r}")
    seconds = int(match.group(1)) * UNIT_SECONDS[match.group(2)]
    return MinPublishAge(timedelta(seconds=seconds), raw)


def load_allowlist() -> set[str]:
    names = set()
    for line in ALLOWLIST.read_text().splitlines():
        name = line.split("#", 1)[0].strip()
        if name:
            names.add(name)
    return names


def crates_io_versions(lock_text: str) -> set[tuple[str, str]]:
    """The (name, version) of every crates.io package in a Cargo.lock."""
    # A Cargo.lock package's `source` value when it comes from crates.io (vs git/path).
    CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"

    packages = tomllib.loads(lock_text).get("package", [])
    return {
        (pkg["name"], pkg["version"])
        for pkg in packages
        if pkg.get("source") == CRATES_IO_SOURCE
    }


def git_show(revision: str, path: str) -> str | None:
    """Contents of `path` at git `revision`, or None if it doesn't exist there."""
    result = subprocess.run(
        ["git", "show", f"{revision}:{path}"],
        cwd=REPO_ROOT, capture_output=True, text=True,
    )
    return result.stdout if result.returncode == 0 else None


def index_url(name: str) -> str:
    """The sparse-index URL for a crate, per the crates.io path layout."""
    n = name.lower()
    if len(n) == 1:
        path = f"1/{n}"
    elif len(n) == 2:
        path = f"2/{n}"
    elif len(n) == 3:
        path = f"3/{n[0]}/{n}"
    else:
        path = f"{n[0:2]}/{n[2:4]}/{n}"
    return f"{INDEX_BASE}/{path}"


# Memoizes pubtimes() so each crate's index is fetched from the network at most once.
_pubtime_cache: dict[str, dict[str, str]] = {}


def pubtimes(name: str) -> dict[str, str]:
    """Map of version -> pubtime for a crate, read from the sparse index."""
    if name not in _pubtime_cache:
        request = urllib.request.Request(index_url(name), headers={"User-Agent": USER_AGENT})
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                body = response.read().decode()
        except urllib.error.URLError as e:
            sys.exit(f"error: failed to fetch crates.io index for {name!r}: {e}")
        entries = (json.loads(line) for line in body.splitlines() if line.strip())
        _pubtime_cache[name] = {e["vers"]: e["pubtime"] for e in entries if e.get("pubtime")}
    return _pubtime_cache[name]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "base_ref",
        nargs="?",
        default=None,
        help="git ref to diff each Cargo.lock against; only crates.io versions "
        "present in the working tree but not in this ref are checked. If omitted, "
        "every crates.io version in the lockfiles is checked.",
    )
    return parser.parse_args()


def main() -> int:
    base_ref = parse_args().base_ref
    min_publish_age = load_min_publish_age()
    if min_publish_age.duration == timedelta(0):
        print("min-publish-age is 0; nothing to check.")
        return 0
    allowlist = load_allowlist()
    # UTC keeps `now` timezone-aware for subtracting the index's UTC pubtimes.
    now = datetime.now(timezone.utc)

    if base_ref is not None and subprocess.run(
        ["git", "rev-parse", "--verify", "--quiet", base_ref],
        cwd=REPO_ROOT, capture_output=True,
    ).returncode != 0:
        sys.exit(f"error: base ref {base_ref!r} is not a valid git ref")

    lockfiles = sorted(subprocess.run(
        ["git", "ls-files", "*Cargo.lock"],
        cwd=REPO_ROOT, capture_output=True, text=True,
    ).stdout.split())

    violations = []
    num_checked_crates = 0
    for lock in lockfiles:
        head_crates = crates_io_versions((REPO_ROOT / lock).read_text())
        # With no base ref, check the whole lockfile; otherwise only the crates.io
        # versions the working tree adds on top of that ref.
        base_ref_crates: set[tuple[str, str]] = set()
        if base_ref is not None:
            base_text = git_show(base_ref, lock)
            base_ref_crates = crates_io_versions(base_text) if base_text else set()
        for name, version in sorted(head_crates - base_ref_crates):
            if name in allowlist:
                continue
            pubtime = pubtimes(name).get(version)
            if pubtime is None:
                # crates.io backfills pubtime on every version, so a miss here is
                # an anomaly (e.g. an index change); fail loudly rather than skip.
                sys.exit(f"error: no publish time for {name} {version} in the "
                         "crates.io index")
            num_checked_crates += 1
            age = now - datetime.fromisoformat(pubtime.replace("Z", "+00:00"))
            if age < min_publish_age.duration:
                violations.append(
                    f"{name} {version} ({lock}): published "
                    f"{age.total_seconds() / 86400:.1f} days ago, minimum is "
                    f"{min_publish_age.text}"
                )

    if violations:
        print("FAIL: found crates.io versions newer than the min-publish-age "
              "cooldown window:\n", file=sys.stderr)
        for violation in violations:
            print(f"  - {violation}", file=sys.stderr)
        print("\nWait until they age past the window, or add the crate to "
              f"{ALLOWLIST.relative_to(REPO_ROOT)}.", file=sys.stderr)
        return 1

    scope = "in the full lockfile" if base_ref is None else f"added since {base_ref!r}"
    print(f"OK: checked {num_checked_crates} crates.io version(s) {scope}; "
          f"none younger than {min_publish_age.text} "
          f"(allowlist: {len(allowlist)} crate(s)).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
