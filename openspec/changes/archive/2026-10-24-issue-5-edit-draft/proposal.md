# Proposal: edit a drafted update

## Why

The update dialog can only post, discard or defer a draft. The wording and the status cannot change,
so a user who wants a different comment must discard the draft and write it in ClickUp by hand.

## What Changes

- `e` in the update dialog opens `$EDITOR` on the draft: the comment, then a `status:` line.
- Saving returns to the dialog with the edited draft, and saves it on the link.
- An empty comment posts only the status. An empty comment and status posts nothing.
- The editor runs on the normal screen, and the board returns after it exits.

## Capabilities

### Modified Capabilities

- `task-start`: a drafted update can be edited before approval.

## Impact

`src/app.rs` (`edit_text`, `parse_edit`, `save_pending`), `src/board.rs`, `README.md`, `tests/draft.rs`.
