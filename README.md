# herdr-clickup

A [herdr](https://herdr.dev) plugin for ClickUp:

- **Board** (`prefix+u`): a tab with your current sprints and every space → folder → list on the left,
  and the selected list's tasks on the right.
  `↑↓`/`jk` move · `⏎` open/expand · `s` write spec · `c` code now · `o` open in browser · `/` filter · `m` only mine ·
  `a` pick an assignee (everyone, me, unassigned, or anyone in the list) ·
  `tab` switch side · `r` refresh · `q` quit.
- **My tasks** (`prefix+shift+u`): a quick fuzzy picker over your open tasks, in a popup.
- **Write spec** (`s`, the default): asks which agent and which repo (repos open in your herdr
  workspaces; last picks first), creates a worktree on `CU-<id>-<slug>`, sets up
  [OpenSpec](https://openspec.dev) there if the repo has none, creates the change `cu-<id>-<slug>` with
  the whole ticket (description, checklists, subtasks, attachments, comments) in its `clickup.md`, and
  has the agent write the proposal, specs, design and tasks, with no code. Review it with
  [herdr-openspec](https://github.com/FlorisKr/herdr-openspec) (`prefix+t`). Needs the `openspec` CLI.
- **Code now** (`c`): the same worktree and agent, but the agent gets the whole ticket and starts
  coding; the task moves to *in progress*.
- If the agent asks something at startup (e.g. "trust this folder?"), answer it in the new
  workspace; the plugin waits and then sends the task. It never answers for you.
- **Report back**: when the agent finishes its turn, comments on the task: "spec drafted" for a spec
  start, or "finished" plus a move to *review* for a code start.
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
| `CLICKUP_STATUS_DONE` | `review` | Status set when a code start's agent is done; empty to skip |
| `CLICKUP_STATUS_SPEC` | (none) | Status set when a spec start begins and when its spec is drafted |

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

`bin/run.sh list` prints your tasks; `bin/run.sh start [--code] <task-id> [agent] [repo]` starts one without the picker (spec first unless `--code`).

## Develop

```sh
cargo test
herdr plugin link .        # after cargo build --release
herdr plugin log list --plugin ssk090.clickup
```

State (pane → task links) lives in `$HERDR_PLUGIN_STATE_DIR/panes.json`.
