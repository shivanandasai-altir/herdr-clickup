# Design: the editor outlives the TUI for a moment

## Decisions

- **Restore the TUI, run the editor, re-enter.** A terminal editor needs the normal screen. The run
  loop restores the screen, runs `$EDITOR`, then calls `ratatui::init()` and redraws. The restore
  happens before the file write, so the caller always re-enters.
- **A text file with a header and a status line.** The first line is dropped on read. The last line
  that starts with `status:` sets the status. Everything before it is the comment. A missing or empty
  status line means no status change.
- **Save on the link.** `save_pending` writes the edited draft to every link to the task, so Esc and
  `u` show it again. The draft is still posted only on Enter.
- **An empty comment is legal.** The post path from #3 already skips an empty comment and sets the
  status. Both empty means nothing to post.
- **A failed or cancelled editor keeps the old draft.** No change is saved when the editor exits
  with a failure or the file cannot be read.

## Risks

- A GUI editor that returns before the file is saved needs `$EDITOR="code -w"` or similar.
- The temporary file name is per task id. Two boards on one machine could collide for one task.
