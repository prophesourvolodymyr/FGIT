# STYLES.md - FGIT Design System

## Existing WTFIS Reference

FGIT must inherit the established WTFIS CLI visual and terminal style from:

`/Users/volodymurvasualkiw/GSpace/Opensource/WTF/WTFIS-CLI`

Before implementing the UI, read the relevant patterns in:

- `/Users/volodymurvasualkiw/GSpace/Opensource/WTF/WTFIS-CLI/src/main.rs`
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTF/WTFIS-CLI/README.md`
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTF/WTFIS-CLI/public/`
- `/Users/volodymurvasualkiw/GSpace/Opensource/WTF/WTFIS-CLI/shell/`

This is a direct product-style reference. FGIT reuses the WTFIS visual
language, system-terminal colors, concise copy, mouse quality, and cleanup
discipline while adapting the workflow to Git commits.

## Product Tone

FGIT belongs to the WTFIS family. It is direct, compact, slightly irreverent,
and useful before it is decorative. The interface should feel like a fast
terminal tool made for a person who already knows what they changed but does
not want to remember Git syntax.

## Terminal Mode

FGIT uses a full-screen Ratatui TUI with the alternate screen. It must not use
an inline viewport for the main workflow. Leaving the TUI returns the terminal
to the shell without a reserved blank area.

## Color Rules

Use terminal system colors by default. The user's terminal theme owns the
actual appearance.

| Meaning | Default treatment |
|---|---|
| Primary accent | System accent / terminal default emphasis |
| Normal content | Terminal default foreground |
| Muted metadata | Dark gray or terminal muted color |
| Selected item | Accent plus bold or inverse emphasis |
| Success | Green semantic color |
| Warning | Yellow semantic color |
| Error | Red semantic color |
| Destructive action | Red label and explicit confirmation |

Do not hardcode a light or dark background. Do not assume a particular ANSI
palette. Explicit user color configuration may be added later.

## Layout Rules

Use bordered panels with clear titles. Keep the active decision visually larger
than secondary metadata. The layout adapts to file count and terminal size.

Small state is optimized for a few changed files. It can use a compact file
grid, a message area, and a destination panel in one view.

Big state is optimized for many changed files. It gives the file list more
space, moves the message and destination into larger stacked panels, and keeps
the primary action visible.

The small and big layouts are responsive variants of the same workflow, not
different products.

## Navigation Rules

Enter advances or activates the focused action. Escape always moves back or
cancels the current modal. `b` is an explicit back shortcut. `q` quits from
the root screen. `h` or `?` opens contextual help. Arrow keys and `j`/`k`
move selection. Mouse clicks activate visible controls.

Do not require a user to press Enter twice to discover how to go back. Every
screen must visibly show its back and next actions.

## Motion Rules

The initial version uses restrained transitions only. A screen transition may
clear and redraw the full alternate screen. Progress updates may refresh the
active row or progress bar. Do not add decorative animation that delays a Git
operation.

## Content Rules

Use plain language first and show the exact Git operation second. For example,
show `Stage selected files`, then optionally show `git add -- file` in a
details panel. Never make the user decode a raw flag to understand an action.

## Mouse Rules

Mouse capture stays enabled while FGIT is open. Clicking files, buttons, tabs,
and navigation controls must work. Mouse capture is released during teardown
so the parent shell receives normal mouse behavior after FGIT exits.
