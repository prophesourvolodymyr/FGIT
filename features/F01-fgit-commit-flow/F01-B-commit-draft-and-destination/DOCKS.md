# F01-B - Commit Draft and Destination

This sub-feature builds the message, commit-type, scope, destination, and final
review states.

## Reference

Use `genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png` for the
message panel, destination panel, small/big layouts, and `Fucking push` stage.
Use `genesis/REFERENCE/FGIT-blueprint/FGIT-CLI-explanation.png` for `-p`, `-r`,
combined selection/destination flags, and custom-message behavior.

Use the WTFIS CLI repository as the direct product-style reference:

`/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI`

Read `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/README.md` for
concise command language and `/Users/volodymurvasualkiw/GSpace/Opensource/WTFIS-CLI/src/main.rs`
for system-terminal colors, focus states, help presentation, and navigation.

## What We Build

FGIT creates a commit draft from the selected changes. The user can choose a
message type, optional scope, optional breaking marker, summary, body, and
trailers. The user can choose local-only or post-commit push behavior.

## Message Rules

The message editor accepts a direct custom message. `-m` and `--message` are
valid CLI forms. Multi-word messages are parsed as one value when quoted by the
shell. FGIT never executes message text as a command.

The default automatic message is `Auto commit`, matching the CLI blueprint.
The default can be changed in configuration later.

The editor has three modes. Auto mode is the default and always previews the
literal `Auto commit` message. Structured mode renders
`type(scope)!: summary`, then appends optional body and trailer paragraphs.
Custom mode is entered by `-m` or `--message` and preserves the supplied
argument exactly as commit-message data. The user may edit type, scope,
breaking marker, summary, body, and trailers in structured mode. Type and
summary are required there; the custom path requires a non-empty message.

`Tab`, arrows, and `j`/`k` move between fields. Normal character input edits
the selected field, Backspace removes its final character, and Space toggles
the breaking marker. `Ctrl+a` switches between Auto and Structured modes so
the ordinary `a` character remains available while writing a message.

## Destination Rules

The destination chooser lists local-only, configured upstream, and available
Git remotes. It must state that all options create the local commit first. A
remote option then runs a separate push operation.

Repository discovery reads `git remote`, plus the current branch's configured
remote and merge ref. Failed or absent upstream lookup is treated as no
upstream; it does not prevent local review. Destination choices are local only,
the configured upstream when present, and each discovered remote. Selecting a
remote only records future intent. F01-B does not call `git add`, `git commit`,
`git push`, hooks, or any other Git write operation.

## Review State

The review screen shows selected files, excluded files, commit message, local
commit action, optional push action, hook policy, and warnings. It shows an
equivalent Git command or command sequence in a details panel.

The implemented review is explicitly non-writing. It displays an argument-safe
visual equivalent of `git add -- <selected paths>` followed by `git commit -m
<message>`, with display-only single-quote escaping. It names the selected
destination separately and labels every remote operation as optional after the
local commit. Enter only acknowledges the future operation and displays that
Git is unchanged. Empty selection remains visible as a warning for F01-C to
reject during execution.

## Implementation

- `src/command.rs` parses only the documented `c`, `com`, and `commit` fast
  paths. It accepts `-p`, `-s`, `-r`, their short combined forms, `-m VALUE`,
  `-mVALUE`, `--message VALUE`, and `--message=VALUE`; all other arguments
  fail before terminal startup. The operating system provides the argument
  vector, so custom message text is never shell-interpolated.
- `src/draft.rs` contains the validated Auto, Structured, and Custom commit
  message model.
- `src/git.rs` extends the read-only repository model with remotes and an
  optional configured upstream.
- `src/app.rs` adds draft, destination, and confirmation states while retaining
  F01-A's file selection, diff, responsive layout, mouse capture, help, back,
  resize, and cleanup behavior.
- `src/main.rs` parses the fast command before opening the alternate screen and
  passes its typed launch options into the application.

## Verification

Verify generated and custom messages, aliases, preview mode, file-selection
mode, destination mode, combined flags, back navigation, Escape, contextual
help, no accidental push, and review accuracy against Git's actual index state.

| Test scenario | Command / fixture | Result and evidence |
|---|---|---|
| Format, compile, unit tests, lint, release build | `cargo fmt --check && cargo test && cargo clippy -- -D warnings && cargo build --release` | Passed on macOS arm64 with 12 library tests and 3 existing real-Git adapter fixtures. |
| Draft model | `draft` unit tests | Auto mode previews literal `Auto commit`; structured mode joins type, optional scope, breaking marker, body, and trailers; custom `-m` text remains verbatim. |
| Fast command parser | `command` unit tests and `./target/release/fgit c -x` | All aliases and documented flags parse from an argument vector without shell evaluation; invalid `-x` exits with a direct parser error before terminal setup. |
| Keyboard, mouse, help, back, resize | Ratatui `TestBackend` app tests | F01-A navigation remained passing. New test navigated selection, draft, destination, remote selection, review, confirmation acknowledgement, and rendered the result without a Git subprocess write. |
| Destination and non-writing review on target terminal | macOS `/usr/bin/script` pseudo-terminal at 120x36, repository fixture with `origin`, `backup`, and `branch.main` upstream | Capture at `/var/folders/f2/k3zmhjfx5yd6x965gw58m3m00000gn/T/opencode/fgit-f01b-runtime-session.log` showed Auto commit, local-only, `origin/main`, remote choices, an optional `backup` push, and the confirmation acknowledgement. `git status --short` was `?? "changed file.txt"` both before and after. |
| Automatic preview path and exact selected-file operation | Same fixture: `fgit c -p` in a macOS pseudo-terminal | Capture at `/var/folders/f2/k3zmhjfx5yd6x965gw58m3m00000gn/T/opencode/fgit-f01b-auto-session.log` showed `git add -- 'changed file.txt'`, `git commit -m 'Auto commit'`, local-only destination, and `F01-C owns execution`. Status was unchanged before and after. Both runtime captures include alternate-screen entry, mouse-capture enablement, all matching mouse-release sequences, and alternate-screen exit. |
