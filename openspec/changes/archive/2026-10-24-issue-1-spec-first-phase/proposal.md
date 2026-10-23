# Proposal: finish spec-first starts

## Why

The plugin can already write an OpenSpec change from a ClickUp ticket, but the board does not show
where a task is, and there is no way to start the implementation after the spec is reviewed. A
finished implementation also reports back as "spec drafted", so the team cannot tell the two apart.

## What Changes

- A pane link records the task's phase: spec drafting, spec drafted, implementing, or done.
- The board shows a PHASE column, and shows the `i` action only when a reviewed change waits to be
  implemented.
- `i` sends the OpenSpec apply workflow to the agent in the task's pane, and moves the task to
  implementing.
- The next finish after `i` drafts an implementation report with the done status. A spec finish
  keeps drafting the spec report with the spec status.
- Links written before this change keep their meaning: a change means the spec phase, no change
  means a code start.

## Capabilities

### New Capabilities

- `task-start`: how a ClickUp task becomes an OpenSpec change, is implemented, and reports back.

### Modified Capabilities

None.

## Impact

`src/hook.rs` (link phase), `src/app.rs` (start and implement), `src/board.rs` (phase column and
`i`), `src/ticket.rs` (apply prompt), and `src/main.rs` (report wording).
