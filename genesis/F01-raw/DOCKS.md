# F01 - FGIT Commit Flow Raw Feature

This raw feature describes the first complete FGIT workflow. It is retained as
the origin record. The promoted feature documentation in `features/` is the
build source of truth after approval.

## What We Build

FGIT opens a full-screen Ratatui TUI from a Git repository. It reads Git state,
lets the user choose changes, collects a commit message, lets the user choose
local or remote follow-up behavior, confirms the operation, runs Git, shows
progress, and reports the commit hash.

## Reference Images

The TUI blueprint at
`genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png` must be used
when documenting and implementing every visual stage. It defines the small
state, big state, navigation question, message panel, destination panel,
progress display, and commit-number result.

The CLI explanation at
`genesis/REFERENCE/FGIT-blueprint/FGIT-CLI-explanation.png` must be used when
documenting the aliases and fast paths. It defines the intent behind `fgit c`,
preview, file selection, destination choice, and custom messages.

## Required States

| State | Entry | Main content | Exit |
|---|---|---|---|
| Repository loading | `fgit` | Detect root and Git state | Ready or error |
| Clean repository | No changes | Explanation and quit | Quit or refresh |
| Small changes | Few files | Compact file/message/destination layout | Next stage or back |
| Big changes | Many files | Expanded file grid and stacked panels | Next stage or back |
| File selection | Select action | Files, statuses, diff preview | Continue or back |
| Message draft | Message action | Message editor and commit preview | Continue or back |
| Destination choice | Destination action | Local, configured remote, chosen remote | Continue or back |
| Final confirmation | Last stage | Exact operation and warning | Execute or back |
| Commit progress | Execute | File/hook/push progress | Result or error |
| Commit result | Success | Hash and remaining changes | Quit or another commit |
| Error | Any Git failure | Plain error, output, recovery actions | Back or quit |
| Conflict blocked | Conflicted repository | Conflict list and explanation | Quit |

## Git Correction

The destination panel describes the operation after the local commit. The local
commit itself always writes to the repository index and object database. The
remote destination is a separate push step. The UI may present both steps as a
single user journey, but the confirmation screen must show both operations.

## Navigation Correction

Enter advances or activates. Escape goes back or cancels. `b` goes back. `q`
quits from the root. `h` or `?` opens help. Arrow keys, `j`, `k`, and mouse
clicks remain available. No screen requires a second Enter to discover how to
leave it.

## Product Style Reference

The TUI must use the established WTFIS CLI style from:

`/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI`

Relevant files to read before implementation are:

- `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/src/main.rs`
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/README.md`
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/public/`
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/shell/`

FGIT inherits the WTFIS system-terminal colors, compact copy, selected-state
emphasis, mouse behavior, and return-to-shell quality. It adapts those patterns
to Git and uses a full-screen alternate-screen workflow.

## Verification

The promoted F01 documentation owns the complete verification matrix. This raw
document must remain a faithful record of the source idea and the two blueprint
images.
