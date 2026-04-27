# Agent Guide

This fork exists to add a small number of community-requested Mullvad daemon features while
remaining easy to update from upstream `mullvad/mullvadvpn-app`.

## Core Rule

Preserve upstream mergeability. Treat every fork change as a small patch that should survive
regular upstream merges with minimal conflict.

## How To Work In This Fork

- Prefer isolated fork modules, folders, and adapters for new behavior.
- Touch upstream files only at narrow import, type, command, or call-site boundaries.
- Avoid broad refactors, formatting-only edits, dependency churn, and unrelated cleanups.
- Keep features daemon-first unless `.agent/backlog.md` explicitly expands the scope.
- Do not claim a feature is implemented until code, tests, and user-facing docs agree.
- Keep commits focused so upstream sync conflicts are easy to inspect and resolve.

## Suggested Feature Shape

When adding a feature, start with a feature note under `.agent/features/` that records:

- User problem
- Intended behavior
- In-scope platforms
- Settings or management-interface changes
- Daemon integration points
- Verification plan

Implementation should usually live in a fork-owned module and be wired into upstream code through
the smallest practical integration points.

## Upstream Sync Mindset

Before editing a file that upstream changes often, ask whether the fork logic can sit beside it
instead. When an upstream file must change, keep the patch local, obvious, and easy to reapply.

## Cross-Platform Guardrails

- Gate Linux-only fork modules, imports, fields, enum variants, and command handlers with
  `#[cfg(target_os = "linux")]` at every shared-code hook.
- Do not export Linux-only CLI modules from shared command trees on macOS or Windows; CI runs with
  `-D warnings`, so unused fork code can fail unrelated platform builds.
- If a value only needs to be mutable on Linux, bind it inside a Linux-only block or destructuring
  pattern instead of making the whole function argument mutable.
- When adding public modules to crates that warn on missing docs, especially `talpid-core`, add
  module and public-item docs before pushing.
- Before committing, scan the touched hooks from a Linux, macOS, and Windows perspective even when
  the feature itself is Linux-only.
