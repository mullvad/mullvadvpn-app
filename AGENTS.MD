# Agent Workflow & Organization

This document defines workflows, organization patterns, and development processes for AI agents working on this Mullvad VPN fork.

## Mission Statement

This fork adds community-requested Mullvad daemon features while remaining easy to update from upstream `mullvad/mullvadvpn-app`. Every decision should support this goal.

## Core Rule

**Preserve upstream mergeability.** Treat every fork change as a small patch that should survive regular upstream merges with minimal conflict.

## Development Workflow

### 1. Feature Planning Phase

Before writing code, create a feature note in `.agent/features/`:

**Required sections:**
- **User problem**: What pain point does this solve?
- **Intended behavior**: How should the feature work?
- **In-scope platforms**: Which OS platforms are supported?
- **Settings/API changes**: What management interface or settings changes are needed?
- **Daemon integration points**: Where does this hook into existing code?
- **Verification plan**: How will we test this works correctly?

**Example structure:**
```markdown
# Feature: IP Split-Tunneling

## User Problem
Users need to allow specific IP ranges to bypass Mullvad so private VPNs 
like Tailscale can work alongside Mullvad.

## Intended Behavior
- Daemon accepts CIDR ranges to whitelist
- Routes for whitelisted IPs bypass Mullvad tunnel
- Leak protection remains active for non-whitelisted traffic

## In-Scope Platforms
- Linux (initial implementation)
- macOS, Windows (future consideration)

## Settings Changes
- Add `ip_whitelist: Vec<IpNetwork>` to settings
- Add management interface commands: `whitelist add`, `whitelist remove`, `whitelist list`

## Daemon Integration Points
- `mullvad-daemon`: settings persistence
- `talpid-routing`: route table manipulation
- `talpid-core/firewall`: firewall rule coordination

## Verification Plan
1. Unit tests for whitelist parsing and validation
2. Integration test: verify whitelisted IP bypasses tunnel
3. Manual test: confirm Tailscale works while Mullvad is connected
```

### 2. Implementation Phase

Follow this sequence:

1. **Create fork-owned modules**
   - Start with isolated modules in appropriate crates
   - Keep fork code self-contained
   - Use clear naming conventions

2. **Add minimal upstream hooks**
   - Touch upstream files only at integration boundaries
   - Keep patches small and obvious
   - Add platform cfg gates immediately

3. **Implement daemon logic first**
   - Settings persistence
   - Management interface commands
   - Core feature behavior
   - Platform-specific implementations

4. **Add tests**
   - Unit tests for fork modules
   - Integration tests for daemon behavior
   - Cross-platform compilation checks

5. **Update documentation**
   - User-facing docs
   - API documentation
   - Feature note (reflect actual implementation)

### 3. Verification Phase

Before claiming completion:

- [ ] Code compiles on all target platforms
- [ ] Tests pass
- [ ] Manual verification plan executed
- [ ] Documentation matches implementation
- [ ] No `-D warnings` issues on any platform
- [ ] Upstream merge conflicts are minimal and obvious

## Organization Patterns

### Feature Scope Management

**Daemon-first approach:**
- Keep features daemon-focused initially
- CLI interface comes after daemon is stable
- GUI interface is lowest priority
- Don't expand scope beyond `.agent/backlog.md` without explicit decision

**Scope boundaries:**
- Do not claim a feature is implemented until code, tests, and docs agree
- If implementation reveals scope issues, update the feature note
- Keep features minimal and focused

### Code Isolation Strategy

**Prefer isolation over integration:**
- Can this be a separate module? → Yes: create fork-owned module
- Must touch upstream file? → Keep patch minimal and obvious
- Need to modify shared types? → Consider adapter pattern instead

**Integration point hierarchy (prefer earlier options):**
1. Import fork module at call site
2. Add optional field to existing struct
3. Add enum variant with cfg gate
4. Modify function signature (last resort)

### Platform Handling

**Linux-only features:**
- Gate all Linux-specific code with `#[cfg(target_os = "linux")]`
- Provide no-op stubs or compile-time exclusion for other platforms
- Test that macOS and Windows builds succeed
- Document platform limitations in feature notes

**Cross-platform features:**
- Implement platform-specific modules under fork namespace
- Use trait abstractions for platform differences
- Test on all supported platforms before merging

## Upstream Sync Workflow

### Before Starting Work

```bash
# Fetch latest upstream
git fetch upstream

# Check for upstream changes in areas you'll modify
git log upstream/main --oneline -- <path-to-files>

# Consider rebasing if upstream has relevant changes
git rebase upstream/main
```

### During Development

- Keep commits small and focused
- Write commit messages that explain fork-specific changes
- Avoid reformatting or refactoring upstream code

### Handling Merge Conflicts

**Conflict resolution priority:**
1. Preserve upstream behavior first
2. Reapply fork hooks second
3. Test that both upstream and fork functionality work

**Conflict resolution process:**
1. Understand what upstream changed and why
2. Identify fork-specific modifications
3. Merge changes preserving both intentions
4. Verify fork features still work
5. Verify upstream behavior is preserved

**Avoid during conflict resolution:**
- Rewriting unrelated upstream files
- Changing upstream code style
- Mixing conflict resolution with feature work

## Patch Hygiene

### What to Stage

**Always stage:**
- Files directly related to current task
- New fork-owned modules
- Minimal upstream integration points
- Updated documentation
- New tests

**Never stage:**
- Unrelated user configuration files
- Generated files (unless intentional)
- Formatting-only changes to upstream files
- Dependency updates unrelated to feature
- Debug code or temporary experiments

### Commit Organization

**Good commit:**
```
Add IP whitelist settings to daemon

- Create fork_ip_whitelist module in mullvad-daemon
- Add whitelist field to Settings with persistence
- Gate Linux-only code with cfg(target_os = "linux")
- Add unit tests for whitelist validation
```

**Bad commit:**
```
Various improvements and fixes

- Add IP whitelist
- Fix formatting
- Update dependencies
- Refactor routing code
- Add comments
```

## Verification Checklist

### For Documentation Changes

- [ ] Staged diffs reviewed
- [ ] No planned features described as implemented
- [ ] Typos and formatting checked
- [ ] Links and references valid

### For Code Changes

- [ ] Compiles on Linux
- [ ] Compiles on macOS (if shared code touched)
- [ ] Compiles on Windows (if shared code touched)
- [ ] Tests pass
- [ ] No dead code warnings
- [ ] No unused mutability warnings
- [ ] No missing documentation warnings
- [ ] No unused import warnings
- [ ] Manual verification completed

### For Fork Features

- [ ] Feature note exists and is accurate
- [ ] Fork code is isolated where practical
- [ ] Upstream integration points are minimal
- [ ] Platform cfg gates are in place
- [ ] Settings persistence works
- [ ] Management interface documented
- [ ] Leak prevention preserved
- [ ] User documentation updated

## Communication Patterns

### When to Update Feature Notes

- Before implementation starts (initial planning)
- When design decisions change during implementation
- After implementation completes (reflect actual behavior)
- When scope changes or limitations discovered

### When to Update Backlog

- When new feature requests emerge
- When current feature completes
- When priorities shift
- When platform support expands

### When to Ask Questions

**Ask before proceeding if:**
- Feature scope is ambiguous
- Platform support is unclear
- Integration approach has multiple options
- Upstream changes conflict with fork design
- Security or leak-prevention implications exist

**Don't ask, just proceed if:**
- Implementation details are clear from feature note
- Standard patterns apply
- Change is isolated to fork-owned code
- Tests and verification plan are straightforward

## Anti-Patterns

### Development Anti-Patterns

- **Big bang implementation**: Don't implement entire feature in one commit
- **Premature optimization**: Don't optimize before basic functionality works
- **Scope creep**: Don't add "nice to have" features without backlog approval
- **Upstream modification**: Don't refactor upstream code to fit fork patterns
- **Platform assumptions**: Don't assume Linux-only code won't affect other builds

### Organization Anti-Patterns

- **Scattered fork code**: Don't spread fork logic across many upstream files
- **Deep integration**: Don't tightly couple fork code to upstream internals
- **Missing documentation**: Don't skip feature notes or verification plans
- **Incomplete features**: Don't merge code that doesn't match documentation
- **Stale documentation**: Don't leave feature notes describing planned vs actual behavior

### Workflow Anti-Patterns

- **Mixing concerns**: Don't combine feature work with refactoring or cleanup
- **Ignoring conflicts**: Don't merge upstream without testing fork features
- **Skipping verification**: Don't skip cross-platform compilation checks
- **Batch commits**: Don't commit multiple unrelated changes together
- **Missing tests**: Don't claim completion without tests

## Quick Reference

### Starting a New Feature

1. Check `.agent/backlog.md` for scope and priority
2. Create feature note in `.agent/features/`
3. Identify daemon integration points
4. Create fork-owned modules
5. Add minimal upstream hooks with cfg gates
6. Write tests
7. Update documentation
8. Verify on all platforms
9. Update feature note to reflect implementation

### Before Every Commit

1. Review staged files (only task-related?)
2. Check compilation on all platforms
3. Run tests for changed code
4. Verify no `-D warnings` issues
5. Confirm documentation matches code
6. Write clear commit message

### Before Claiming Completion

1. Code, tests, and docs all agree
2. Manual verification plan executed
3. Feature note reflects actual implementation
4. All platforms build successfully
5. No warnings on any platform
6. Upstream merge conflicts would be minimal

## Resources

- **Feature planning**: `.agent/features/README.md`
- **Current priorities**: `.agent/backlog.md`
- **Upstream sync**: `.agent/workflows/README.md`
- **Coding standards**: `CLAUDE.MD`
- **Project conventions**: `rustfmt.toml`, `clippy.toml`, `CONTRIBUTING.md`
