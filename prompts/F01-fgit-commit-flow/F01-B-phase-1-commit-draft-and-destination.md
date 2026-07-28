# Phase 1 of F01-B - Commit Draft and Destination

## Features Included

- F01-B - Commit Draft and Destination

F01-B must build, test, verify, and check off its own Cycle 2 items. Do not
implement F01-C execution, hooks, or push progress in this phase.

## Context

F01-A is complete: it provides a verified full-screen selection TUI, direct
Git status adapter, lazy focused diffs, and terminal cleanup. This phase adds a
validated commit draft and clearly separates the local commit decision from an
optional later push decision.

## What You Need to Read First

- `AGENTS.md`
- `STYLES.md`
- `CYCLES.md`
- `features/F01-fgit-commit-flow/DOCKS.md`
- `features/F01-fgit-commit-flow/F01-A-repository-state-and-selection/DOCKS.md`
- `features/F01-fgit-commit-flow/F01-B-commit-draft-and-destination/DOCKS.md`
- `genesis/REFERENCE/FGIT-blueprint/REFERENCE.md`
- `genesis/REFERENCE/FGIT-blueprint/FGIT-TUI-UI-UX-blueprint.png`
- `genesis/REFERENCE/FGIT-blueprint/FGIT-CLI-explanation.png`

## What Happened Last Session

F01-A created `src/git.rs`, `src/app.rs`, and `src/main.rs`. It was verified on
macOS with real Git fixtures for clean, mixed staged/unstaged, untracked,
deleted, renamed, conflict, and large-diff states; its evidence is in the
F01-A DOCKS verification table.

## What to Build

- Add the message editor with type, scope, breaking marker, body, trailers, and
  an explicit `Auto commit` message path.
- Add local-only, configured-upstream, and choose-remote destination states.
  Explain that commit is local first and a push is an optional later operation.
- Add safe parsing for `fgit c`, `fgit com`, `fgit commit`, `-p`, `-s`, `-r`,
  their combined forms, and `-m` / `--message` as data rather than shell input.
- Add an exact operation preview and confirmation state without running any
  Git write operation. F01-C owns execution, hooks, and result reporting.
- Preserve F01-A navigation, mouse capture, responsive layout, and cleanup.

## Files to Create or Modify

- modify: `Cargo.toml`
- modify: `src/main.rs`
- modify: `src/app.rs`
- modify: `src/git.rs`
- create or modify: focused parser and draft-model modules under `src/`
- modify: `CYCLES.md`
- modify: `features/F01-fgit-commit-flow/F01-B-commit-draft-and-destination/DOCKS.md`

## Verification

- [ ] formatting, compilation, unit tests, and lints pass
- [ ] message validation and automatic message preview are visible
- [ ] destination choices retain local-first semantics
- [ ] every documented fast command parses without shell interpolation
- [ ] confirmation shows the exact future operation without changing Git state
- [ ] keyboard, mouse, help, back, and alternate-screen cleanup work on macOS
- [ ] evidence captures terminal, fixture, commands, visible result, and Git state before and after

## Agent Rules (Mandatory - Do Not Skip)

1. **NO SUB-AGENTS:** Do not spawn sub-agents to review this repository. Do all work yourself in this session.
2. **COMMIT AFTER DONE:** After completing every task, commit with a clear message describing what was built.
3. **UPDATE CYCLES.md:** After verifying a task, mark it `[x]` in `CYCLES.md`. This is the user's progress dashboard.
4. **NO IDE TODO SYSTEMS:** Do not create IDE-specific todo/task JSON or duplicate task lists. All project todos live in `CYCLES.md`.
5. **COMPLETE THE WHOLE SLICE:** Build, test, fix, verify, update `CYCLES.md`, and commit within this session. Do not create a handoff only for testing or obvious bug fixes.
6. **GENERATE THE NEXT PROMPT ONLY IF NEEDED:** Before ending, evaluate the remaining work against the 550k context limit. Create the next prompt only if it is a self-contained, substantial 250-550k implementation slice. The next prompt must state this session's verification, the next task and paths, decisions, and unresolved notes, and save under `prompts/F01-fgit-commit-flow/`.

## When You Finish

Report what was built, what was verified, what evidence was captured, and
whether a next prompt was needed.

## Scope Decision Before Finish

- Completed: F01-B draft, destination, parser, confirmation, verification, docs, and commit.
- Remaining: F01-C commit execution, hooks, push execution, and result state.
- Next-scope estimate: 300k context.
- Decision: create a next prompt only after F01-B is verified.
