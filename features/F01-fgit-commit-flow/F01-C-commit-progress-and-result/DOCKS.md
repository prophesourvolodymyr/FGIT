# F01-C - Commit Progress and Result

This sub-feature connects the reviewed draft to Git execution and reports the
result without losing the user's work.

## Reference

Use `genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png` for the
`Fucking push` confirmation, per-file progress concept, completed state,
commit number, and extra details. The image's per-file progress is interpreted
as operation progress: staging, hooks, and push can report progress, while the
local commit itself is atomic.

Use the WTFIS CLI repository as the direct terminal lifecycle and cleanup
reference:

`/Users/volodymurvasualkiw/GSpace/Opensource/WTF/WTFIS-CLI`

Read `/Users/volodymurvasualkiw/GSpace/Opensource/WTF/WTFIS-CLI/src/main.rs` for
mouse capture teardown, system color handling, terminal restoration, and
return-to-shell behavior. FGIT uses a full-screen alternate screen, but the
same cleanup quality is required.

## What We Build

FGIT runs Git with direct arguments, displays hook output and push progress,
reports local commit success separately from push success, and returns the
terminal to the shell cleanly.

The confirmation, progress, and result states occupy the full terminal rather
than appearing as nested forms. They use short, high-contrast ASCII headings:
`F U C K I N G PUSH` for the commit confirmation and progress stage, followed by
`C O M M I T` and the short hash on the result stage. The progress bar animates
through three visible frames before Git execution begins; the result keeps only
the compact local/push status words and exit controls visible.

## Result States

| State | Content |
|---|---|
| Preparing | Selected operation summary |
| Running hooks | Hook name, output, and elapsed state |
| Creating commit | Local commit operation in progress |
| Commit success | Short hash, full message, file count |
| Pushing | Remote, branch, progress, output |
| Push success | Remote destination and final status |
| Push failure | Local hash preserved, remote error shown |
| Hook failure | No commit claimed, hook output and retry path |
| Cancelled | No false success message |
| Terminal cleanup | Alternate screen closed and mouse capture released |

## Safety Rules

No automatic force push exists in the first feature. Hook bypass is advanced
and visibly warned. A local commit that succeeds before a push failure must be
shown as successful locally. FGIT must not erase staged or unstaged changes.

## Verification

Verify successful local commit, successful local-plus-push flow, hook failure,
push failure after commit, cancellation, Ctrl+C, mouse cleanup, terminal
cleanup, final hash display, remaining changes, and repeat commit from the
success screen.

| Test scenario | Result and evidence |
|---|---|
| Formatting, unit tests, Git adapter tests, lint, release build | Passed locally on macOS with `cargo fmt && cargo test && cargo clippy -- -D warnings && cargo build --release`. Ten unit tests and three real-Git adapter tests passed. |
| Blueprint workflow rendering | Ratatui `TestBackend` verifies both small and big responsive layout selection and renders the selection-to-confirmation flow. |
| Local commit operation | `RepositoryState::commit_selected` stages only the selected paths using direct Git argument arrays, creates a local commit, and reads the short `HEAD` hash. Empty selections and Git failures return an error state. |
| Optional push operation | Upstream and explicitly selected remote destinations run only after local commit success. Push errors are preserved on the success screen as "local commit succeeded, but push failed." |
| Target-terminal visual review | Pending: run `target/release/fgit` in the intended terminal and compare visual density, panel proportions, and mouse navigation against the supplied blueprint. |
