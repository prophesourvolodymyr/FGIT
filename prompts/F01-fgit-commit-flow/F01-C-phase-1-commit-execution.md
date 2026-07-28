# Phase 1 of F01-C - Commit Execution and Result

## Features Included

- F01-C - Commit Progress and Result

## Context

F01-A selection and F01-B draft, destination, safe CLI parsing, and
non-writing review are verified. This phase turns only the confirmed reviewed
operation into direct Git argument-array execution and reports local and push
results independently.

## What You Need to Read First

- `AGENTS.md`
- `STYLES.md`
- `CYCLES.md`
- `features/F01-fgit-commit-flow/DOCKS.md`
- `features/F01-fgit-commit-flow/F01-A-repository-state-and-selection/DOCKS.md`
- `features/F01-fgit-commit-flow/F01-B-commit-draft-and-destination/DOCKS.md`
- `genesis/REFERENCE/FGIT-blueprint/REFERENCE.md`
- `src/app.rs`
- `src/git.rs`
- `src/command.rs`
- `src/draft.rs`

## What Happened Last Session

F01-B added `CommitDraft`, command parsing, read-only remote/upstream discovery,
draft and destination screens, and a confirmation preview. It was verified on
macOS with a real fixture: review shows exact future commands but Enter does
not write Git state. F01-C must preserve that local-first distinction.

## What to Build

- Execute the confirmed staging and local commit using direct Git argument arrays.
- Run hooks through Git, show progress and captured failures, and never retry a
  destructive operation automatically.
- Report local commit hash and remaining changes after a successful commit.
- Run the optional push as a separately reported post-commit operation.
- Preserve local success when push fails, including the local commit hash and
  actionable push error.
- Add fixture tests for hook success, hook failure, local commit success, and
  push failure after a successful local commit.

## Files to Create or Modify

- modify: `src/app.rs`
- modify: `src/git.rs`
- modify: `src/main.rs` if terminal lifecycle needs execution-specific repair
- modify: `CYCLES.md`
- modify: `features/F01-fgit-commit-flow/DOCKS.md`
- create or modify: focused execution/result module under `src/`
- modify: `features/F01-fgit-commit-flow/F01-B-commit-draft-and-destination/DOCKS.md` only if its verified F01-B interface requires a documented adjustment

## Verification

- [ ] formatting, compilation, unit tests, integration fixtures, and lints pass
- [ ] local commit runs with direct Git arguments and reports its short hash
- [ ] hook success and failure are visibly distinct
- [ ] optional push runs after successful local commit only
- [ ] failed push preserves visible local success and Git history
- [ ] Ctrl+C, panic, mouse capture, and alternate-screen cleanup work on macOS
- [ ] evidence captures terminal, fixture, commands, visible result, Git status before and after

## Agent Rules (Mandatory - DO NOT SKIP)

1. **NO SUB-AGENTS:** Do not spawn sub-agents to review this repository. Do all work yourself in this session.
2. **COMMIT AFTER DONE:** After completing every task, commit with a clear message describing what was built.
3. **UPDATE CYCLES.md:** After verifying a task, mark it `[x]` in `CYCLES.md`. This is the user's progress dashboard.
4. **NO IDE TODO SYSTEMS:** Do not create IDE-specific todo/task JSON or duplicate task lists. All project todos live in `CYCLES.md`.
5. **COMPLETE THE WHOLE SLICE:** Build, test, fix, verify, update `CYCLES.md`, and commit within this session. Do not create a handoff only for testing or obvious bug fixes.
6. **GENERATE THE NEXT PROMPT ONLY IF NEEDED:** Before ending, evaluate remaining work against the 550k context limit. Create the next prompt only if it is a self-contained, substantial 250-550k implementation slice. Include what was built and verified, direct DOCKS paths, decisions, and unresolved notes.

## When You Finish

Report what was built, what was verified, what evidence was captured, and
whether a next prompt was needed.

## Scope Decision Before Finish

- Completed: direct execution, result states, verification, docs, and commit.
- Remaining: no planned F01 work unless verification finds a defect.
- Next-scope estimate: determine from remaining defects only.
- Decision: no next prompt unless a substantial post-F01 slice remains.
