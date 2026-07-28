# F01 Raw Idea - FGIT Commit Flow

FGIT should make committing a small edit feel simple. The user opens a full
TUI, selects the files, writes a message, chooses local or remote behavior,
confirms, and sees progress and the commit number.

The interface has a small state for a few files and a big state for massive
changes. The same actions must remain available in both layouts.

The CLI has a normal full-TUI command and short automatic commit commands. The
user wants `fgit c`, `fgit com`, and `fgit commit` to converge on the same fast
commit behavior, with preview, selection, destination, and custom-message
options available when needed.

The source screenshots are:

- `genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png`
- `genesis/REFERENCE/FGIT-blueprint/FGIT-CLI-explanation.png`
