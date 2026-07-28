# F01-A - Repository State and Selection

This sub-feature builds the first screen and the complete changed-file model.

## Reference

Use `genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png` as the
visual authority. Its small state and big state show how the file grid adapts.
Use `genesis/REFERENCE/FGIT-blueprint/FGIT-CLI-explanation.png` for the `-s`
file-selection command path.

Use the WTFIS CLI repository as the direct style and terminal-behavior
reference:

`/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI`

Read `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/src/main.rs` for
system colors, selected-row emphasis, mouse capture, and terminal lifecycle.

## What We Build

FGIT detects the repository root with Git, reads branch and status data, loads
changed paths, distinguishes staged and unstaged content, and renders the file
selection state. It supports keyboard and mouse selection, diff inspection, and
the same behavior in small and big layouts.

## States

| State | Behavior |
|---|---|
| Loading | Show repository path and a short loading message |
| Clean | Show that no commitable changes exist |
| Modified | Show modified path and staged/unstaged state |
| Added | Show new path and whether it is staged |
| Deleted | Show deletion status and diff availability |
| Renamed | Show old and new path when Git reports both |
| Untracked | Show untracked marker and explicit staging choice |
| Conflict | Show conflict marker and disable commit |
| Large diff | Show truncation notice and lazy loading option |
| Terminal too small | Show resize instruction without corrupting layout |
| Git error | Show captured stderr and retry/quit actions |

## Selection Rules

Space toggles the focused file. `a` selects all safe visible changes. `u`
selects tracked changes only. `n` clears the commit selection. Enter opens the
focused diff or activates the focused button according to focus context. Escape
returns to the previous state. `b` returns to the previous state explicitly.

Untracked and ignored files must not be staged silently. Ignored files are not
shown in the normal list unless the user requests them.

## Diff Rules

The focused file's diff appears beside or below the file grid. Staged and
unstaged diffs are clearly labeled. Large diffs are loaded lazily and truncated
with a visible limit notice so they cannot block the first screen.

## Implementation

- `Cargo.toml` defines the Rust 2024 binary using Ratatui and Crossterm.
- `src/git.rs` reads repository root, branch, and porcelain-v2 status with direct
  Git argument arrays. It keeps index and worktree status separate and loads
  only the focused file's staged, unstaged, or untracked diff on demand.
- `src/app.rs` owns the loading, clean, selection, full-diff, help, Git-error,
  and terminal-too-small render states. It chooses a side-by-side small layout
  for up to six files at sufficient dimensions and a stacked big layout
  otherwise.
- `src/main.rs` owns raw mode, alternate screen, cursor, and mouse-capture
  lifecycle. Its idempotent cleanup runs on normal quit, Escape, Ctrl+C,
  terminal errors, and stack unwinding.
- F01-A deliberately does not run `git add`, `git commit`, `git push`, or any
  branch, stash, merge, or rebase operation.

## Verification

The sub-feature is verified only when the file list, statuses, small layout,
big layout, keyboard actions, mouse actions, diff preview, clean state, conflict
state, and terminal resize state work on the target Mac terminal.

| Test scenario | Command / fixture | Result and evidence |
|---|---|---|
| Formatting, build, tests, lints | `cargo fmt && cargo test && cargo clippy -- -D warnings && cargo build --release` | Passed on macOS arm64, Rust 1.95.0 and Apple Git 2.50.1. Five unit tests and three real-Git fixture tests passed. |
| Clean repository | `reads_clean_and_all_changed_path_categories` fixture before modifications | `RepositoryState::load` returned an empty file list. |
| Modified, staged, unstaged, untracked, deleted, renamed | Same real-Git fixture after a staged then further-modified file, an untracked file, a deletion, and a staged rename | Test passed. The mixed file reported independent index/worktree `Modified` status; the renamed path retained its old path. |
| Conflicted repository | `reads_conflicts_without_modifying_the_repository` creates divergent `main` and `topic` edits then merges | Test passed. Conflict status was detected and the conflicted working file remained present. The TUI labels conflicts and disables their selection. |
| Large focused diff | `lazily_limits_a_large_untracked_diff` fixture with 40,000 lines | Test passed. The diff was requested only for the focused path, then limited to 48 KiB or 600 lines with a visible truncation notice. |
| Small and big layouts | Ratatui `TestBackend` at 100x30 with 3 and 7 files | Test passed. Three files used the compact split layout; seven used the stacked big layout. |
| Keyboard, mouse, help, back, quit, resize | Ratatui `TestBackend` interaction test | Test passed for arrows, `j`/`k`, Space, `a`/`u`/`n`, Enter, Escape, `b`, `q`, `h`, `?`, file-checkbox mouse click, and resize to 40x10 without panic. |
| Full-screen Mac runtime and teardown | macOS `/usr/bin/script` pseudo-terminal: `printf 'q' \| script -q /var/folders/f2/k3zmhjfx5yd6x965gw58m3m00000gn/T/opencode/fgit-big-session.log sh -c "stty cols 120 rows 36 && exec ./target/release/fgit"` | Captured loading state and a 12-file stacked big layout. The capture contains `EnterAlternateScreen`, all enabled mouse modes, `q`, every matching mouse disable sequence, and `LeaveAlternateScreen`; the command returned normally. |
