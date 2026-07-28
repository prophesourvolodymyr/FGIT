# FGIT - Original Idea

FGIT, or Fucking Git, is a second WTF application. It solves the problem of
wanting to commit a small change without remembering Git prefixes, staging
commands, commit flags, or push syntax.

The tool should be dead simple and blazingly fast. Running `fgit` opens a full
screen visual TUI. The user sees the changed files, writes a commit message,
chooses where the result should go, confirms, and sees the commit result.

The tool also has a simplified CLI. `fgit c`, `fgit com`, and `fgit commit`
are aliases for the fast commit path. The fast path can automatically include
changes and use the configured automatic message and destination policy. A
preview flag, a file-selection flag, a destination flag, and a custom-message
form must remain available.

The original visual blueprint is preserved in:

- `genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png`
- `genesis/REFERENCE/FGIT-blueprint/FGIT-CLI-explanation.png`

These two screenshots are the human-adjusted visual source of truth for the
first feature. The final implementation may correct Git semantics where the
drawings simplify them, but it must preserve the user's incentive: make the
common commit action fast without making the careful path confusing.
