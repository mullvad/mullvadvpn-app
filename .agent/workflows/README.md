# Workflows

## Upstream Sync

- Fetch upstream before planning large changes.
- Rebase or merge upstream with fork patches kept small and reviewable.
- Resolve conflicts by preserving upstream behavior first, then reapplying fork hooks.
- Avoid rewriting unrelated upstream files during conflict resolution.

## Patch Hygiene

- Keep fork code isolated where practical.
- Stage only files related to the current task.
- Leave unrelated user or generated files untouched.
- Use small commits with direct messages.

## Verification

- Run the narrowest useful checks for the files changed.
- For documentation-only changes, verify staged diffs and confirm docs do not describe planned
  features as implemented.
- For daemon changes, include tests or a concrete manual verification path in the feature note.
