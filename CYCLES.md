# CYCLES.md - FGIT
**Planned with user:** 2026-07-27

## Dependency Chain

F01-A Repository State and Selection
    |
    v
F01-B Commit Draft and Destination
    |
    v
F01-C Commit Progress and Result

The project contains one product feature with three sequential sub-features.
Cycle 0 is the required documentation bootstrap. The three product cycles are
Cycles 1, 2, and 3.

## Cycle 0 - Documentation Bootstrap

- [x] Import the project-management scaffold into `HTF/FGIT`
- [x] Move `TUI  UI:UX.png` to `genesis/REFERENCE/FGIT-blueprint/`
- [x] Move `CLI explanation.png` to `genesis/REFERENCE/FGIT-blueprint/`
- [x] Capture the original idea in `genesis/ORIGINAL IDEA.md`
- [x] Define the initial product structure in `genesis/INITIAL IDEA.md`
- [x] Create the F01 raw feature record
- [x] Create the F01 promoted feature and three sub-feature DOCKS.md files
- [x] Define terminal style rules in `STYLES.md`
- [x] Create the Cycle 1 implementation prompt

## Cycle 1 - Repository State and Selection

- [x] F01-A - Detect repository root, branch, Git status, and changed paths
- [x] F01-A - Build full-screen alternate-screen TUI shell
- [x] F01-A - Render small state for a few changed files
- [x] F01-A - Render big state for many changed files
- [x] F01-A - Select and unselect files with keyboard and mouse
- [x] F01-A - Show staged, unstaged, untracked, deleted, renamed, and conflict states
- [x] F01-A - Show focused file diff and safe empty/error states
- [x] F01-A - Verify terminal resize and cleanup on Mac

## Cycle 2 - Commit Draft and Destination

- [x] F01-B - Build message editor and automatic `Auto commit` message
- [x] F01-B - Add type, scope, breaking marker, body, and trailer fields
- [x] F01-B - Implement local-only, configured-remote, and remote chooser states
- [x] F01-B - Implement `fgit c`, `fgit com`, and `fgit commit` aliases
- [x] F01-B - Implement `-p`, `-s`, `-r`, combined flags, and `-m`
- [x] F01-B - Build exact operation preview and confirmation
- [x] F01-B - Verify back, Escape, help, mouse, and arrowless navigation

## Cycle 3 - Commit Progress and Result

- [x] F01-C - Run local commit with direct Git arguments
- [ ] F01-C - Run hooks and display hook output and failure state
- [x] F01-C - Display local commit hash and remaining changes
- [x] F01-C - Run optional push as a separate operation
- [x] F01-C - Display push success separately from local commit success
- [x] F01-C - Preserve local success when push fails
- [x] F01-C - Verify Ctrl+C, panic cleanup, mouse cleanup, and alternate-screen exit
- [ ] F01-C - Verify end-to-end behavior on the target Mac terminal
