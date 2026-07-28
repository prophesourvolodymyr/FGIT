# FGIT - Initial Product Structure

## Product

FGIT is a local Rust and Ratatui Git commit assistant. It wraps Git commands
with a visual workflow that explains the decisions a user normally has to
remember at the shell.

## Core Promise

The user should be able to move from changed files to a correct commit with
three understandable decisions:

1. Which changes belong together?
2. What should the commit say?
3. Should the new local commit remain local or be pushed to a configured remote?

## Required Product Shape

The project begins with one feature, F01, containing three sequential
sub-features. This is intentionally small enough for a maximum of two or three
people to work on without splitting the product into unrelated systems.

## F01 Sub-Features

### F01-A - Repository State and Selection

Detect the repository, branch, changed files, staged state, untracked state,
conflicts, and small versus big layout. Let the user select files and inspect
their diff.

### F01-B - Commit Draft and Destination

Collect the message, expose the user's fast commit shortcuts, and clarify the
destination. A commit is always created locally first. A remote destination is
an optional push action after the local commit succeeds.

### F01-C - Commit Progress and Result

Run the selected Git operations, show hook and push progress, display the final
commit hash, report remaining changes, and return cleanly to the shell.

## Human Blueprint Rules

The small and big states shown in the TUI blueprint are mandatory responsive
variants. The `Fucking push` screen is retained as the final destination and
confirmation stage, but its wording must explain that Git commits locally and
pushes remotely as two sequential operations.

The CLI explanation screenshot is mandatory for the fast command surface. Its
commands are normalized into valid, explicit forms in F01 documentation.

## Corrected Git Assumptions

`git commit` writes to the local repository. It does not commit directly to a
remote repository. FGIT must preserve the user's remote-destination idea by
performing a local commit followed by an optional push.

`fgit c` may be automatic, but it must follow the configured local and push
policy. A remote push is never silently performed unless the user enabled that
policy.

`fgit c --message text` is normalized to `fgit c -m "text"`. The parser may
also accept a quoted multi-word message, but it must never interpret arbitrary
trailing text as an unsafe shell command.

## Reference Assets

The implementation must repeatedly consult the two files in
`genesis/REFERENCE/FGIT-blueprint/`. The TUI screenshot governs screen
composition, small/big state behavior, navigation, progress, and result
presentation. The CLI screenshot governs aliases, preview, file selection,
destination selection, and automatic commit intent.
