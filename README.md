# herdr-clickup

A [herdr](https://herdr.dev) plugin for ClickUp:

- **Board** (`prefix+u`): a tab with your current sprints and every space → folder → list on the left,
  and the selected list's tasks on the right.
  `↑↓`/`jk` move · `⏎` open/expand · `s` write spec · `i` implement a drafted spec · `c` code now ·
  `A` add a repo to a running task · `X` close a task (asks first) · `u` review a drafted update (✉) ·
  `o` open in browser · `/` filter · `m` only mine ·
  the PHASE column shows `spec…`, `spec ✓`, `code…` or `done` ·
  `a` pick an assignee (everyone, me, unassigned, or anyone in the list) ·
  `tab` switch side · `r` refresh · `q` quit.
- **My tasks** (`prefix+shift+u`): a quick fuzzy picker over your open tasks, in a popup.
- **Write spec** (`s`, the default): asks which agent and which repos (space toggles; repos open in
  your herdr workspaces, last picks first), creates a worktree on `CU-<id>-<slug>`, sets up
  [OpenSpec](https://openspec.dev) there if the repo has none, creates the change `cu-<id>-<slug>` with
  the whole ticket (description, checklists, subtasks, attachments, comments, custom fields, parent) in its `clickup.md`, and
  has the agent write the proposal, specs, design and tasks, with no code. Review it with
  [herdr-openspec](https://github.com/FlorisKr/herdr-openspec) (`prefix+t`), edit `repos.txt` to the
  repos the change affects, then press `i` to have the same agent implement it. Needs the `openspec` CLI.
- **Two or more repos**: one task folder `~/.herdr/tasks/CU-<id>-<slug>/` with a worktree per repo on
  the same branch, one AGENTS.md, one workspace and one agent. A spec start creates only the repo that
  holds the change; `i` adds the worktrees that `repos.txt` lists. One repo behaves as before.
- **Code now** (`c`): the same worktree (or task folder) and agent, but the agent gets the whole
  ticket and starts coding; a move to *in progress* is drafted for your approval.
- **Add a repo** (`A`) to a running multi-repo task, and **close a task** (`X`) to remove every
  worktree and its folder. Close asks first and lists repos with uncommitted changes.
- If the agent asks something at startup (e.g. "trust this folder?"), answer it in the new
  workspace; the plugin waits and then sends the task. It never answers for you.
- **Every ClickUp write is a draft, only with your approval**: a start drafts its status change
  ("in progress", or the spec status), and a finish drafts a comment plus a status ("spec drafted" for a
  spec start; "implementation done" plus a move to *review* for a code start). **Nothing is posted
  automatically.** The drafted comment lists the branch's commits and changed files for code, or the
  proposal's Why and task count for a spec. On the board the task gets a ✉; press `u` to read the exact
  comment and status, `⏎` to post it, `d` to discard it, `esc` to decide later. A status-only draft posts
  no comment.
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
| `CLICKUP_STATUS_STARTED` | `in progress` | Status drafted when a code start begins; empty to skip |
| `CLICKUP_STATUS_DONE` | `review` | Status drafted when a code start's agent is done; empty to skip |
| `CLICKUP_STATUS_SPEC` | (none) | Status drafted when a spec start begins and when its spec is drafted |

A status is never set directly. The plugin drafts it, and posts it only when you approve the draft.

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

State (pane → task links: branch, phase, repos) lives in `$HERDR_PLUGIN_STATE_DIR/panes.json`.
Multi-repo task folders live in `~/.herdr/tasks/`.

## Specs

This repo uses [OpenSpec](https://openspec.dev). Specs live in `openspec/specs`, and each change
starts as a folder in `openspec/changes` with a proposal, spec deltas and tasks.

```sh
openspec list                 # changes in flight
openspec validate --all       # every change and spec
openspec archive <change>     # after merge: apply the deltas to openspec/specs
```
