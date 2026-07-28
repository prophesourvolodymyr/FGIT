# F01 - FGIT Commit Flow

FGIT is a fast full-screen Rust and Ratatui interface for selecting changes,
writing a commit message, choosing local or remote follow-up behavior, and
creating a Git commit without requiring the user to remember Git syntax.

## What We Build

The feature owns the complete first-use flow from `fgit` launch through local
commit result. It also owns the fast CLI forms that enter the same operation.

The feature does not become a general Git client. Branch creation, merge,
rebase, stash management, and conflict resolution are outside this feature.

## Source of Truth

The human-adjusted visual source is
`genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png`.

The human-adjusted CLI source is
`genesis/REFERENCE/FGIT-blueprint/FGIT-CLI-explanation.png`.

The written reference index is
`genesis/REFERENCE/FGIT-blueprint/REFERENCE.md`.

The visual blueprint must be consulted for the small state, big state,
navigation, destination, progress, and result screens. The CLI blueprint must
be consulted for every fast command form.

## WTFIS Product Style Reference

FGIT must use the established WTFIS CLI style from:

`/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI`

Before building the UI, read:

- `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/src/main.rs` for TUI
  layout, system colors, mouse capture, and terminal lifecycle patterns.
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/README.md` for product
  voice, command presentation, and visual language.
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/public/` for existing
  visual assets and presentation patterns.
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/shell/` for shell
  integration and clean return-to-shell behavior.

This is a direct style reference. FGIT keeps the WTFIS branding, system
terminal colors, concise labels, selected states, mouse support, and cleanup
quality while changing the product flow to Git.

## Architecture

```text
CLI parser
    -> repository detector
    -> Git status reader
    -> F01 state machine
         -> file selection and diff view
         -> commit draft and destination view
         -> confirmation and execution
         -> progress and result view
    -> Git subprocess adapter
    -> terminal cleanup
```

The application uses Rust, Ratatui, and direct Git subprocess calls. It must
use argument arrays rather than shell interpolation or `eval`.

The main TUI uses the alternate screen and restores the terminal on normal
exit, Escape, Ctrl+C, Git failure, panic, and terminal resize failure.

## User Flow

The user runs `fgit` from inside a repository. FGIT detects the repository root,
branch, upstream, changed files, staged files, untracked files, conflicts, and
ahead/behind state. It chooses the small or big layout based on the amount of
visible work and terminal dimensions.

The user selects files or hunks. The diff preview updates to the focused file.
The user enters a message. The user chooses whether the resulting local commit
should remain local or be followed by a push to a configured or selected remote.
The final screen shows the exact local commit and optional push operations.
The user confirms. FGIT runs hooks, creates the local commit, optionally pushes,
shows progress, shows the short commit hash, and reports remaining changes.

## Fast CLI Forms

| Form | Behavior |
|---|---|
| `fgit` | Open the complete full-screen TUI |
| `fgit c` | Automatic commit path using configured policy |
| `fgit com` | Alias of `fgit c` |
| `fgit commit` | Alias of `fgit c` |
| `fgit c -p` | Preview automatic commit without changing Git state |
| `fgit c -s` | Select files, then use automatic commit message and policy |
| `fgit c -r` | Choose post-commit destination policy |
| `fgit c -s -r` | Select files and choose post-commit destination |
| `fgit c -m "message"` | Use an explicit custom commit message |
| `fgit c --message "message"` | Long form of custom message |

The automatic path may stage and commit all changes only when the user has
enabled automatic staging. The default destination is local. Remote push is
never silently added merely because a remote exists.

## Git Semantics

The local commit and remote push are separate operations. FGIT can present them
as one simple journey, but it must report both operations and their failures
separately. A successful local commit followed by a failed push must not be
reported as a failed commit.

## States

| State | Visual behavior | Primary actions |
|---|---|---|
| Loading | Full-screen loading title and repository path | Wait, quit |
| Clean | No file grid; explain there is nothing to commit | Refresh, quit |
| Small | Compact grid plus message and destination panels | Select, message, destination |
| Big | Expanded file grid plus stacked message and destination panels | Select, message, destination |
| File selection | Focused file, status, diff preview | Select, inspect, continue |
| Message | Focused message field and generated preview | Edit, continue, back |
| Destination | Local, configured remote, selected remote options | Choose, continue, back |
| Confirmation | Exact operation, warning, file count, message | Commit, back, cancel |
| Progress | Current stage, hooks, files, push progress | Wait, cancel where safe |
| Success | Large short hash, message, remaining changes | Quit, another commit |
| Error | Git error and captured output | Retry, back, quit |
| Conflict | Commit disabled and conflict paths shown | Quit, inspect status |

## Navigation

Enter moves forward or activates the focused control. Escape moves back one
screen or cancels a modal. `b` moves back without relying on arrow keys. `q`
quits from the root. `h` and `?` open contextual help. Arrows and `j`/`k` move
focus. Space toggles file selection. Mouse clicks activate the same visible
controls as keyboard actions.

## Small State

The small state is used when the changed-file set fits comfortably beside the
message and destination panels. The file grid remains the primary visual
element. The message and destination controls remain visible so the user can
complete the flow without opening extra screens.

The small state must be based on measured terminal width and height. It must
not use fixed coordinates. If the terminal becomes too small, it falls back to
the big state's stacked layout or a clear resize warning.

## Big State

The big state is used for many changed files or a large diff. The file area gets
the largest region. The message panel and destination panel remain available
below or beside it. The focused file owns the diff preview. Per-file progress
appears only during actual staging, hook, or push work; Git commit creation is
atomic and must not pretend to commit each file separately.

## Destination Correction

The destination panel offers:

| Choice | Meaning |
|---|---|
| Local only | Create the commit and stop |
| Configured remote | Create locally, then push to the configured upstream |
| Choose remote | Create locally, then choose a Git remote and branch |

The panel title may retain the blueprint's `Fucking push` voice, but the
subtitle must explain `commit locally, then push remotely`.

## Error Handling

FGIT stops before commit when no repository exists, Git is unavailable, the
repository has unresolved conflicts, or the selected state cannot be read.

FGIT preserves staged and unstaged changes when a hook or push fails. It shows
the local commit hash if commit succeeded before push failed. It never retries a
destructive action automatically.

## Dependencies

- F01-A must establish repository state and selection before F01-B.
- F01-B must produce a validated commit draft before F01-C.
- F01-C connects the verified selection and draft into Git execution.
- Rust and Ratatui are implementation dependencies.
- Git is a required runtime dependency.

## Files

The implementation will create or modify the Rust project files after the
documentation cycles are approved. The first code prompt must not add branch,
stash, merge, or rebase features.

## Verification

Verification must cover clean, modified, staged, unstaged, untracked, deleted,
partially staged, conflicted, detached-HEAD, no-upstream, ahead, behind, and
diverged states. It must cover both small and big layouts, keyboard navigation,
mouse clicks, Escape cleanup, Ctrl+C cleanup, terminal resize, hook success,
hook failure, successful local commit, failed push after successful commit, and
custom message parsing.

Evidence must record the target Mac terminal, repository fixture, exact command,
visible result, Git status before and after, and any screenshot or captured
output. The two blueprint screenshots must be checked during visual review.
