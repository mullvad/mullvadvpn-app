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

A version whose publish time cannot be determined fails the check too; an unknown
age is not a safe age. It is reported next to the too-new ones at the end of the
run, so a crate the index cannot answer for does not hide the rest of the
findings.

The cooldown window lives only in .cargo/config.toml (registry.global-min-publish-age).
This script reads it from there, so there is a single source of truth.

Hopefully cargo develops native functionality that can replace this script eventually.
"""

import argparse
import http.client
import json
import re
import ssl
import subprocess
import sys
import time
import tomllib
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import NamedTuple

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
CONFIG = REPO_ROOT / ".cargo" / "config.toml"
ALLOWLIST = SCRIPT_DIR / "rust-min-publish-age-allowlist.txt"
# A Cargo.lock package's `source` value when it comes from crates.io (vs git/path).
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
INDEX_HOST = "index.crates.io"
USER_AGENT = "mullvadvpn-app min-publish-age check (github.com/mullvad/mullvadvpn-app)"
FETCH_TIMEOUT_SECONDS = 30
# The index is a CDN, so failures are usually transient. Retry a few times with a
# growing delay before giving up on a crate.
FETCH_ATTEMPTS = 3
FETCH_RETRY_DELAY_SECONDS = 2

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


class Violation(NamedTuple):
    """A crates.io version that is younger than the cooldown window."""
    name: str
    version: str
    lockfiles: list[str]  # the lockfiles it was found in
    published: datetime
    age: timedelta


class Undatable(NamedTuple):
    """A crates.io version whose publish time could not be established."""
    name: str
    version: str
    lockfiles: list[str]  # the lockfiles it was found in
    reason: str


def load_min_publish_age() -> MinPublishAge:
    with CONFIG.open("rb") as config_file:
        raw = tomllib.load(config_file).get("registry", {}).get("global-min-publish-age")
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


def crates_io_versions(lockfile_text: str) -> set[tuple[str, str]]:
    """The (name, version) of every crates.io package in a Cargo.lock."""
    packages = tomllib.loads(lockfile_text).get("package", [])
    return {
        (package["name"], package["version"])
        for package in packages
        if package.get("source") == CRATES_IO_SOURCE
    }


def git(*args: str) -> subprocess.CompletedProcess[str]:
    """Run a git command in the repo root. What a failure means is up to the caller."""
    return subprocess.run(["git", *args], cwd=REPO_ROOT, capture_output=True, text=True)


def git_show(revision: str, path: str) -> str | None:
    """Contents of `path` at git `revision`, or None if it doesn't exist there."""
    result = git("show", f"{revision}:{path}")
    return result.stdout if result.returncode == 0 else None


def tracked_lockfiles() -> list[str]:
    """Every Cargo.lock tracked by git, as paths relative to the repo root.

    Exits on failure rather than returning nothing: an empty list would make the
    whole check silently pass.
    """
    result = git("ls-files", "*Cargo.lock")
    if result.returncode != 0:
        sys.exit(f"error: listing lockfiles failed: {result.stderr.strip()}")
    lockfiles = sorted(result.stdout.splitlines())
    if not lockfiles:
        sys.exit(f"error: no Cargo.lock is tracked by git in {REPO_ROOT}")
    return lockfiles


def index_path(name: str) -> str:
    """The sparse-index path for a crate, per the crates.io path layout."""
    lowercase_name = name.lower()
    if len(lowercase_name) == 1:
        return f"/1/{lowercase_name}"
    elif len(lowercase_name) == 2:
        return f"/2/{lowercase_name}"
    elif len(lowercase_name) == 3:
        return f"/3/{lowercase_name[0]}/{lowercase_name}"
    else:
        return f"/{lowercase_name[:2]}/{lowercase_name[2:4]}/{lowercase_name}"


class CratesIoIndexError(Exception):
    """A crate version could not be dated from the crates.io index."""


def parse_index(document: str, name: str) -> dict[str, str]:
    """Map of version -> pubtime from an index document (one JSON object per line).

    The name in the document is checked against the crate that was asked about:
    all requests share one connection, so a response must never be read as an
    answer about the wrong crate.
    """
    version_pubtimes = {}
    for line in document.splitlines():
        if not line.strip():
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError as e:
            raise CratesIoIndexError(f"malformed crates.io index: {e}") from e
        # The index path is lowercased, so the document spells the name as published.
        if entry.get("name", "").casefold() != name.casefold():
            raise CratesIoIndexError(
                f"crates.io index document is for {entry.get('name')!r}, not {name!r}"
            )
        if entry.get("vers") and entry.get("pubtime"):
            version_pubtimes[entry["vers"]] = entry["pubtime"]
    return version_pubtimes


def request_document(connection: http.client.HTTPSConnection, name: str) -> str:
    """One attempt at reading a crate's index document.

    Raises CratesIoIndexError if crates.io says the crate does not exist, and
    HTTPException, OSError or UnicodeDecodeError if the request failed in a way
    worth retrying.
    """
    connection.request("GET", index_path(name), headers={"User-Agent": USER_AGENT})
    response = connection.getresponse()
    # The body has to be read in full before the connection can carry the next request.
    body = response.read()
    if response.status == 404:
        # A missing crate is an answer, not a hiccup; retrying cannot help.
        raise CratesIoIndexError("not found in the crates.io index")
    if response.status != 200:
        # Redirects are deliberately not followed: the index path layout is
        # fixed, so a 3xx is an anomaly to report, not something to chase.
        raise http.client.HTTPException(f"HTTP {response.status} {response.reason}")
    return body.decode()


def pubtimes(connection: http.client.HTTPSConnection, name: str) -> dict[str, str]:
    """Map of version -> pubtime for a crate, retrying transient failures.

    Raises CratesIoIndexError if the crate does not exist, if the index answers
    about a different crate, or if every attempt failed.
    """
    last_error: Exception | None = None
    for attempt in range(1, FETCH_ATTEMPTS + 1):
        try:
            # A CratesIoIndexError from either call is deliberately not retried:
            # a missing crate, a malformed document, or a document about another
            # crate are all answers that asking again would not change.
            return parse_index(request_document(connection, name), name)
        # OSError covers socket, TLS and timeout errors. HTTPException covers a
        # response that arrives damaged, such as a body shorter than its
        # Content-Length. A body truncated mid-character fails to decode, which
        # is just as transient.
        except (OSError, http.client.HTTPException, UnicodeDecodeError) as e:
            # A failed request can leave the connection unusable. Closing it is
            # enough; the next request opens a new one by itself.
            connection.close()
            last_error = e
            print(f"warning: {name}: index request failed ({e}); "
                  f"attempt {attempt} of {FETCH_ATTEMPTS}", file=sys.stderr)
        if attempt < FETCH_ATTEMPTS:
            time.sleep(FETCH_RETRY_DELAY_SECONDS * attempt)
    raise CratesIoIndexError(
        f"failed to fetch the crates.io index in {FETCH_ATTEMPTS} attempts: {last_error}"
    )


def published_at(crate_pubtimes: dict[str, str], version: str) -> datetime:
    """When a version was published, as a timezone-aware datetime.

    Raises CratesIoIndexError if the index has no usable publish time for it. An
    unknown or ambiguous timestamp cannot be compared against the window, and an
    unknown age is not a safe age.
    """
    pubtime = crate_pubtimes.get(version)
    if pubtime is None:
        # crates.io backfills pubtime on every version, so a miss here is an
        # anomaly (e.g. an index change) rather than something to skip.
        raise CratesIoIndexError("version has no publish time in the crates.io index")
    try:
        published = datetime.fromisoformat(pubtime)
    except ValueError as e:
        raise CratesIoIndexError(f"unparsable publish time {pubtime!r}") from e
    if published.tzinfo is None:
        raise CratesIoIndexError(f"publish time {pubtime!r} has no time zone")
    return published


def versions_to_check(lockfiles: list[str], base_ref: str | None,
                      allowlist: set[str]) -> dict[str, dict[str, list[str]]]:
    """Crate name -> version -> the lockfiles that version was found in.

    Grouped by crate name because one index request answers for all of its
    versions, and lockfiles overlap heavily.

    With a base_ref, only the versions the working tree adds on top of that git
    ref are included. With None, every crates.io version in the lockfiles is.
    """
    crates: dict[str, dict[str, list[str]]] = {}
    for lockfile in lockfiles:
        versions = crates_io_versions((REPO_ROOT / lockfile).read_text())
        if base_ref is not None:
            # A lockfile that does not exist at the base ref is entirely new, so
            # every version in it gets checked.
            base_text = git_show(base_ref, lockfile)
            if base_text is not None:
                versions -= crates_io_versions(base_text)
        for name, version in versions:
            if name not in allowlist:
                crates.setdefault(name, {}).setdefault(version, []).append(lockfile)
    return crates


def format_age(age: timedelta) -> str:
    """An age as whole days and hours, e.g. "2d 3h".

    A negative age, meaning a publish time in the future, keeps its sign rather
    than being hidden behind wrapped-around numbers.
    """
    sign = "-" if age < timedelta(0) else ""
    hours = int(abs(age).total_seconds() // 3600)
    return f"{sign}{hours // 24}d {hours % 24}h"


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

    if base_ref is not None:
        if git("rev-parse", "--verify", "--quiet", base_ref).returncode != 0:
            sys.exit(f"error: base ref {base_ref!r} is not a valid git ref")

    lockfiles = tracked_lockfiles()
    crates = versions_to_check(lockfiles, base_ref, allowlist)

    # One connection for every request: a fresh TLS session per crate costs several
    # times more than transferring the small index documents does.
    index_connection = http.client.HTTPSConnection(
        INDEX_HOST,
        timeout=FETCH_TIMEOUT_SECONDS,
        # The default TLS context: certificate chain and hostname are both verified.
        context=ssl.create_default_context(),
    )
    violations: list[Violation] = []
    undatable: list[Undatable] = []
    num_checked_versions = 0
    for name, versions in sorted(crates.items()):
        # One index request answers for every version of a crate.
        try:
            crate_pubtimes = pubtimes(index_connection, name)
        except CratesIoIndexError as e:
            for version, version_lockfiles in sorted(versions.items()):
                undatable.append(Undatable(name, version, version_lockfiles, str(e)))
            continue
        for version, version_lockfiles in sorted(versions.items()):
            try:
                published = published_at(crate_pubtimes, version)
            except CratesIoIndexError as e:
                undatable.append(Undatable(name, version, version_lockfiles, str(e)))
                continue
            num_checked_versions += 1
            age = now - published
            if age < min_publish_age.duration:
                violations.append(
                    Violation(name, version, version_lockfiles, published, age))
    index_connection.close()

    if violations:
        print("FAIL: found crates.io versions newer than the min-publish-age "
              "cooldown window:\n", file=sys.stderr)
        for violation in sorted(violations, key=lambda entry: entry.age):
            print(f"  - {violation.name} {violation.version} "
                  f"({', '.join(violation.lockfiles)}): published "
                  f"{violation.published:%Y-%m-%d %H:%M}Z, "
                  f"{format_age(violation.age)} ago, minimum is {min_publish_age.text}",
                  file=sys.stderr)
        print("\nWait until they age past the window, or add the crate to "
              f"{ALLOWLIST.relative_to(REPO_ROOT)}.", file=sys.stderr)

    if undatable:
        if violations:
            print(file=sys.stderr)  # separate the two lists
        print(f"FAIL: could not determine the publish time of {len(undatable)} "
              "version(s). An unknown age is not a safe age:\n", file=sys.stderr)
        for entry in undatable:
            print(f"  - {entry.name} {entry.version} ({', '.join(entry.lockfiles)}): "
                  f"{entry.reason}", file=sys.stderr)

    if violations or undatable:
        return 1

    scope = (f"across {len(lockfiles)} lockfile(s)" if base_ref is None
             else f"added since {base_ref!r}")
    print(f"OK: checked {num_checked_versions} crates.io version(s) {scope}; "
          f"none younger than {min_publish_age.text} "
          f"(allowlist: {len(allowlist)} crate(s)).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
