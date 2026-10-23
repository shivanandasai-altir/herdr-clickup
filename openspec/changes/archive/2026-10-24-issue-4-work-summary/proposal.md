# Proposal: a finish draft says what happened

## Why

The drafted finish comment is generic: "herdr agent finished its turn on branch CU-...". It tells the
team nothing about the work. The reviewer must open the branch to learn what changed.

## What Changes

- A code finish draft includes the branch's commits and the changed-file summary.
- A multi-repo finish groups those facts by repo.
- A spec finish draft includes the proposal's Why and how many tasks are done.
- The summary runs in the hook, outside the state lock, and still only produces a draft. The user
  approves every post.

## Capabilities

### Modified Capabilities

- `task-start`: a finish draft summarises the work.

## Impact

`src/report.rs` (new), `src/hook.rs`, `src/main.rs`, `README.md`.
