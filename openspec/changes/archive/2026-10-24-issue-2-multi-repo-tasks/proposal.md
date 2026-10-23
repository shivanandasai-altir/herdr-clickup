# Proposal: one task across several repos

## Why

A start creates a worktree in one repo. Many tickets change a UI repo and a backend repo together,
and one worktree cannot hold both.

## What Changes

- The repo step becomes a multi-select: space toggles a repo, Enter starts.
- Two or more repos share one task folder `~/.herdr/tasks/<branch>/`, with one worktree per repo on
  the same branch, one `AGENTS.md` (`CLAUDE.md` for Claude), one workspace and one agent.
- One repo keeps today's single worktree.
- A spec start creates only the repo that holds the OpenSpec change. The change gets a `repos.txt`
  that lists the repos the task may touch; the user edits it during review.
- `i` creates worktrees for exactly the repos in `repos.txt` before it sends the apply workflow.
- The board shows the branch and repos, adds a repo to a running task (`A`), and closes a task (`X`)
  after a warning that lists repos with uncommitted changes.

## Capabilities

### New Capabilities

- `multi-repo-tasks`: a task that spans repos, from start through implementation and close.

### Modified Capabilities

None.

## Impact

`src/taskspace.rs` (new), `src/app.rs`, `src/board.rs`, `src/hook.rs`, `src/herdr.rs`, `src/main.rs`.
