# Proposal: draft the start status change

## Why

A code start sets `CLICKUP_STATUS_STARTED` as soon as the agent starts. It is the last ClickUp write
that does not wait for approval. A spec start can also set its status directly. Every other write is
a draft on the board.

## What Changes

- A start no longer writes a ClickUp status.
- When a start status is configured, the start puts a status-only draft on the task link. The board
  shows the task's ✉ and `u` opens the draft.
- Approving a status-only draft sets the status and posts no empty comment.
- The README settings table says that a status is a draft.

## Capabilities

### Modified Capabilities

- `task-start`: a start drafts its status change instead of writing it.

## Impact

`src/app.rs`, `src/hook.rs`, `src/board.rs`, `README.md`.
