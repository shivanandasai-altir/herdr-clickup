# Proposal: guard the implement action

## Why

Review found two ways the implement action can fail. A second `i` press sends the apply workflow twice
and can clear the working flag that the hook needs to report the finish. Pressing `i` while the spec
agent is still working also clears that flag, so the next finish is never reported. The action also
trusts that the change validates, though the plugin can check.

## What Changes

- Run `openspec validate --strict` before sending the apply workflow.
- Claim the link only when it is `spec drafted` and the agent is idle, inside the state lock.
- Refuse `i` while the agent works, and make a second press a no-op.
- Store the task's worktree folder on the link, so later actions can find the change.

## Impact

`src/app.rs`, `src/hook.rs`, `src/openspec.rs`, `src/board.rs`, `tests/hook.rs`.
