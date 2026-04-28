# Claude Coding Rules

This document defines coding standards, architecture principles, and technical constraints for working on this Mullvad VPN fork.

## Core Architecture Principle

**Preserve upstream mergeability.** Every change should be a minimal, isolated patch that survives regular upstream merges with minimal conflict.

## Code Organization

### Fork-Owned Modules

- Create new fork-specific modules in isolated folders
- Use clear naming that distinguishes fork code from upstream (e.g., `fork_*`, `custom_*`)
- Keep fork modules self-contained with minimal dependencies on upstream internals
- Wire fork modules into upstream code through narrow, well-defined integration points

### Upstream File Modifications

- Touch upstream files only at:
  - Import boundaries
  - Type definitions (when absolutely necessary)
  - Command registration points
  - Call sites for fork functionality
- Avoid:
  - Broad refactors
  - Formatting-only edits
  - Dependency version changes unrelated to fork features
  - Unrelated cleanups or "improvements"

## Platform-Specific Code

### Cross-Platform Guardrails

All Linux-only fork code must be properly gated:

```rust
#[cfg(target_os = "linux")]
```

Apply `cfg` gates at:
- Module declarations
- Import statements
- Struct fields
- Enum variants
- Command handlers
- Function definitions

### Platform Build Safety

- Do not export Linux-only CLI modules from shared command trees on macOS or Windows
- CI runs with `-D warnings`, so unused code will fail builds on other platforms
- If a value only needs to be mutable on Linux, bind it inside a Linux-only block instead of making the entire function argument mutable
- Before committing, mentally scan touched code from Linux, macOS, and Windows perspectives

### Documentation Requirements

When adding public modules to crates that warn on missing docs (especially `talpid-core`):
- Add module-level documentation
- Document all public items (functions, structs, enums, traits)
- Keep docs concise but clear about purpose and usage

## Rust Code Standards

### General Principles

- Follow existing project conventions (see `rustfmt.toml`, `clippy.toml`)
- Prefer explicit types over inference when it improves clarity
- Use descriptive variable names that match domain terminology
- Keep functions focused and single-purpose

### Error Handling

- Use `Result` types for operations that can fail
- Provide context with error messages
- Avoid panics in daemon code; prefer graceful degradation
- Log errors at appropriate levels

### Testing

- Write unit tests for fork-owned modules
- Include integration tests for daemon behavior changes
- Document manual verification steps in feature notes
- Test cross-platform compilation even for platform-specific features

## Daemon Architecture

### Feature Integration Points

When adding daemon features:

1. **Settings Layer** (`mullvad-daemon`)
   - Add settings fields with sensible defaults
   - Implement persistence
   - Add validation logic

2. **Management Interface** (`mullvad-management-interface`)
   - Define protobuf messages for new commands
   - Keep API changes backward-compatible when possible
   - Document API changes

3. **Core Logic** (`talpid-core`)
   - Implement feature behavior in fork-owned modules
   - Hook into tunnel lifecycle at appropriate points
   - Respect existing leak-prevention mechanisms

4. **Platform Layers** (`talpid-routing`, `talpid-core/src/firewall`)
   - Add platform-specific implementations
   - Gate with appropriate `cfg` attributes
   - Provide no-op stubs for unsupported platforms

### Daemon-First Development

- Start with daemon implementation before UI
- Verify daemon behavior through CLI or management interface
- Keep features testable without GUI dependencies
- Document daemon behavior in feature notes

## Commit Hygiene

### Commit Scope

- Keep commits focused on a single logical change
- Stage only files related to the current task
- Leave unrelated user or generated files untouched
- Avoid mixing feature work with formatting or refactoring

### Commit Messages

- Use clear, direct messages
- Start with a verb (Add, Fix, Update, Remove)
- Reference feature notes or issues when applicable
- Keep messages concise but informative

## Verification Before Commit

### Minimal Checks

Run the narrowest useful checks for changed files:

```bash
# Format check
cargo fmt --check

# Clippy on changed crates
cargo clippy --package <crate-name> -- -D warnings

# Build check for affected platforms
cargo build --target <platform-triple>

# Run tests for changed modules
cargo test --package <crate-name>
```

### Documentation-Only Changes

- Verify staged diffs
- Confirm docs do not describe planned features as implemented
- Check for typos and formatting

### Code Changes

- Verify compilation on all affected platforms
- Run relevant tests
- Check for `-D warnings` issues:
  - Dead code
  - Unused mutability
  - Missing documentation
  - Unused imports
- Follow manual verification plan from feature note

## Feature Implementation Checklist

Before claiming a feature is implemented:

- [ ] Code is written and compiles on all target platforms
- [ ] Tests are written and passing
- [ ] User-facing documentation is updated
- [ ] Feature note reflects actual implementation
- [ ] Management interface changes are documented
- [ ] Settings persistence works correctly
- [ ] Leak-prevention mechanisms are preserved
- [ ] Cross-platform cfg gates are in place
- [ ] Manual verification plan is executed

## Anti-Patterns to Avoid

- **Broad refactoring**: Don't restructure upstream code to fit fork patterns
- **Premature abstraction**: Don't create abstractions until patterns emerge
- **Dependency churn**: Don't update dependencies unless required for fork features
- **Feature creep**: Don't expand scope beyond what's in `.agent/backlog.md`
- **Incomplete features**: Don't merge code that doesn't match its documentation
- **Platform assumptions**: Don't assume Linux-only code won't affect other platforms

## When in Doubt

1. Check if the change can be isolated to a fork-owned module
2. Ask whether upstream would need to change this file often
3. Consider if the integration point is the smallest practical one
4. Verify the change won't complicate upstream merges
5. Confirm all platforms can build with the change
