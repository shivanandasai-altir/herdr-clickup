# herdr-clickup

A [herdr](https://herdr.dev) plugin for ClickUp:

- **Board** (`prefix+u`): a tab with your current sprints and every space → folder → list on the left,
  and the selected list's tasks on the right.
  `↑↓`/`jk` move · `⏎` open/expand · `s` start · `o` open in browser · `/` filter · `m` only mine ·
  `a` pick an assignee (everyone, me, unassigned, or anyone in the list) ·
  `tab` switch side · `r` refresh · `q` quit.
- **My tasks** (`prefix+shift+u`): a quick fuzzy picker over your open tasks, in a popup.
- **Start**: asks which coding agent to use (the installed ones herdr supports: claude, codex, pi, …;
  your last pick comes first), creates a worktree on `CU-<id>-<slug>`, starts that agent in it with the
  task as its first prompt, and moves the task to *in progress*.
- **Report back**: when that agent finishes (`done`), comments on the task and moves it to *review*.
- **Links**: Ctrl-click an `app.clickup.com/t/...` link in a terminal to open the task.

## Install

Needs `cargo`.

```sh
herdr plugin install ssk090/herdr-clickup
# or, from a checkout:
cargo build --release && herdr plugin link .
```

Put your ClickUp personal token (Settings → Apps) in the plugin config dir:

```sh
cat > "$(herdr plugin config-dir ssk090.clickup | tail -1)/.env" <<'EOF'
CLICKUP_TOKEN=pk_...
EOF
```

| Key | Default | |
|---|---|---|
| `CLICKUP_TOKEN` | (required) | Personal API token |
| `CLICKUP_TEAM_ID` | first workspace | ClickUp workspace (team) id |
| `CLICKUP_SPACE_IDS` | all spaces | Comma-separated space ids the board shows (the number after `/s/` in a space URL) |
| `CLICKUP_AGENT` | claude | Agent offered first until you pick one |
| `CLICKUP_STATUS_STARTED` | `in progress` | Status set on start; empty to skip |
| `CLICKUP_STATUS_DONE` | `review` | Status set when the agent is done; empty to skip |

Status names are per ClickUp space; a status that does not exist is skipped with a warning
(see `herdr plugin log list --plugin ssk090.clickup`).

Bind keys in `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+u"
type = "plugin_action"
command = "ssk090.clickup.board"
description = "ClickUp board"

[[keys.command]]
key = "prefix+shift+u"
type = "plugin_action"
command = "ssk090.clickup.tasks"
description = "ClickUp: my tasks (quick picker)"
```

A sprint is any list with start and due dates; *Current sprints* shows those running today.

Run it from a workspace inside the git repo the tasks belong to: the worktree is made from that
workspace's repository.

## CLI

`bin/run.sh list` prints your tasks; `bin/run.sh start <task-id> [agent]` starts one without the picker.

## Develop

```sh
cargo test
herdr plugin link .        # after cargo build --release
herdr plugin log list --plugin ssk090.clickup
```

State (pane → task links) lives in `$HERDR_PLUGIN_STATE_DIR/panes.json`.
