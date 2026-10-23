# Design: task phases

## Context

A pane link already stores the task, branch, change, and the drafted update. The hook turns herdr
agent events into ClickUp drafts. The board reads the state file every second.

## Decisions

- **The phase lives on the link, not in a separate store.** The hook and the board already share
  the link, and one task has one pane. No new file, no extra ClickUp call.
- **The phase is optional.** Old `panes.json` files have no phase. A change means the spec phase:
  drafting until its report was written, then drafted. No change means a code start.
- **`i` changes the link before it prompts the agent.** A fast status event then cannot arrive
  before the link expects an implementation, which would lose the finish.
- **The board reads the phase from the state file.** It already reloads the file every second for
  drafted updates, so the phase costs no extra read or HTTP call.
- **Spec and implementation reports use different statuses.** `CLICKUP_STATUS_SPEC` for a drafted
  spec, `CLICKUP_STATUS_DONE` for a finished implementation. Both stay drafts until approval.

## Risks

- A user who edits `panes.json` by hand can set any phase. The state file is private to the plugin.
- The board shows one phase per task. A later multi-repo start will need one phase per task, not per
  worktree, which this model already supports.
