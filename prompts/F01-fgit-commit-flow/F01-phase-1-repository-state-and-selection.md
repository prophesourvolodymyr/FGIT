# Phase 1 of F01 - Repository State and Selection

## Features Included

F01-A - Repository State and Selection.

This phase builds and verifies the first sub-feature only. Do not implement
message composition, destination pushing, branch management, stash, merge,
rebase, or other Git features in this phase.

## Context

FGIT is a new Rust and Ratatui project. Its human-adjusted source of truth is
the two screenshots in `genesis/REFERENCE/FGIT-blueprint/`. The first phase
must make the repository's changed files understandable and selectable in a
full-screen TUI.

## What You Need to Read First

- `AGENTS.md`
- `STYLES.md`
- `CYCLES.md`
- `features/DOCKS.md`
- `features/F01-fgit-commit-flow/DOCKS.md`
- `features/F01-fgit-commit-flow/F01-A-repository-state-and-selection/DOCKS.md`
- `genesis/INITIAL IDEA.md`
- `genesis/REFERENCE/FGIT-blueprint/REFERENCE.md`
- `genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png`
- `genesis/REFERENCE/FGIT-blueprint/FGIT-CLI-explanation.png`
- `protocols/P01 - Feature Audit.md`

## What Happened Last Session

The project-management scaffold was initialized in the correct target
directory. The two screenshots supplied by the user were moved into the
reference folder. The F01 documentation defines one feature with three
sequential sub-features and records the distinction between local commit and
remote push.

## What to Build

Create the initial Rust project using the project's established Rust and
Ratatui approach. Use a full-screen alternate-screen TUI, not an inline
viewport. Add a direct Git subprocess adapter for repository root, branch, and
machine-readable status.

Build the F01-A state model for loading, clean, modified, staged, untracked,
deleted, renamed, conflict, Git error, large diff, and terminal-too-small
states. Do not claim completion for a state that is not visibly handled.

Build the small and big layouts from the reference image. The small layout is
for a few changed files and keeps the file grid, diff, and primary controls
compact. The big layout gives the file grid more room and uses stacked panels.
Use measured terminal dimensions rather than fixed coordinates.

Add focused file selection with keyboard and mouse. Space toggles a file. Arrow
keys and `j`/`k` move focus. Enter opens the diff or activates the focused
control. Escape goes back. `b` goes back explicitly. `q` quits from the root.
`h` and `?` open contextual help.

Render staged, unstaged, untracked, deleted, renamed, and conflict indicators.
Keep staged and unstaged concepts separate. Do not stage files or create a
commit in this phase.

Add a focused diff preview with safe truncation for large diffs. The diff must
be loaded lazily for the focused file and must not block startup by loading the
whole repository diff.

Ensure mouse capture is enabled while the TUI runs and is released on every
exit path. Ensure the alternate screen and raw mode are restored on Escape,
Ctrl+C, Git error, terminal error, and normal quit.

## Files to Create or Modify

- Create the Rust project manifest and source structure needed for F01-A.
- Create the Git repository/status adapter.
- Create the TUI event loop and alternate-screen terminal lifecycle.
- Create the F01-A state model and small/big renderers.
- Create the file selection and diff preview behavior.
- Modify `CYCLES.md` with verification evidence after testing.
- Modify the relevant F01-A DOCKS.md verification table with evidence.

Do not modify the source project-management repository. Do not read or use the
personal board. The only visual references are the two files in this project's
`genesis/REFERENCE/FGIT-blueprint/` directory.

## Verification

Run formatting, compilation, unit tests, and target Mac runtime checks.

Verify a clean repository fixture.

Verify modified tracked files.

Verify staged and unstaged changes in the same file.

Verify an untracked file.

Verify a deleted file.

Verify a renamed file if the fixture can produce one.

Verify a conflicted repository blocks the commit flow without deleting data.

Verify the small layout with a few changed files.

Verify the big layout with many changed files.

Verify keyboard selection, arrowless navigation, Enter, Escape, `b`, `q`,
`h`, `?`, and mouse clicks.

Verify the focused diff appears and large diff content is safely limited.

Verify terminal resize does not panic or corrupt the screen.

Verify the alternate screen closes and the shell receives normal mouse behavior
after exit.

Record the exact Mac terminal, commands, fixture state, visible result, and
captured evidence in the F01-A DOCKS.md file. Mark only the verified Cycle 1
items `[x]` in `CYCLES.md`.

## Agent Rules

1. Do all work in this session without spawning sub-agents.
2. Build the complete F01-A slice, test it, fix failures, and verify it.
3. Do not implement F01-B or F01-C early.
4. Do not create duplicate IDE task lists.
5. Update `CYCLES.md` and the F01-A DOCKS.md with evidence.
6. Commit the verified implementation with a clear message.
7. Before finishing, decide whether the next prompt is needed. If the next
   slice is substantial, create the Cycle 2 prompt under the same prompt folder.
