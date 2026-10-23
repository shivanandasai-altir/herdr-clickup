# Design: a start writes nothing

## Decisions

- **Draft on the link at start.** The link already holds `pending`, and the board already reads it
  every second. The start puts the status draft there before the agent starts, so the ✉ can appear
  at once. No new state, no extra write.
- **An empty comment means status only.** `post_update` skips the comment and sets the status. The
  update dialog says "Status only, no comment."
- **One draft per task.** A finish overwrites an unapproved start draft with its own. The finish is
  the newer information, and the user approves one update at a time.
- **No status means no draft.** An empty or absent setting leaves the link's `pending` empty.

## Risks

- A user who starts a task and never opens the board keeps the old ClickUp status. That is the point
  of the change.
- The start summary says that the status is drafted, so the user knows to approve it.
