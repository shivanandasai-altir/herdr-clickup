# herdr-clickup

A [herdr](https://herdr.dev) plugin for ClickUp:

- **My tasks**: a fuzzy picker over your open ClickUp tasks, in a popup.
- **Start**: creates a worktree on `CU-<id>-<slug>`, starts an agent in it with the task as its first
  prompt, and moves the task to *in progress*.
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
| `CLICKUP_AGENT` | `claude` | Any `herdr agent start --kind` value |
| `CLICKUP_STATUS_STARTED` | `in progress` | Status set on start; empty to skip |
| `CLICKUP_STATUS_DONE` | `review` | Status set when the agent is done; empty to skip |

Status names are per ClickUp space; a status that does not exist is skipped with a warning
(see `herdr plugin log list --plugin ssk090.clickup`).

Bind the picker to a key in `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+u"
type = "plugin_action"
command = "ssk090.clickup.tasks"
description = "ClickUp tasks"
```

Run it from a workspace inside the git repo the tasks belong to: the worktree is made from that
workspace's repository.

## CLI

`bin/run.sh list` prints your tasks; `bin/run.sh start <task-id>` starts one without the picker.

## Develop

```sh
cargo test
herdr plugin link .        # after cargo build --release
herdr plugin log list --plugin ssk090.clickup
```

State (pane → task links) lives in `$HERDR_PLUGIN_STATE_DIR/panes.json`.
