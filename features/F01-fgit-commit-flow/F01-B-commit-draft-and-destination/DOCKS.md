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

## Destination Rules

The destination chooser lists local-only, configured upstream, and available
Git remotes. It must state that all options create the local commit first. A
remote option then runs a separate push operation.

## Review State

The review screen shows selected files, excluded files, commit message, local
commit action, optional push action, hook policy, and warnings. It shows an
equivalent Git command or command sequence in a details panel.

## Verification

Verify generated and custom messages, aliases, preview mode, file-selection
mode, destination mode, combined flags, back navigation, Escape, contextual
help, no accidental push, and review accuracy against Git's actual index state.
