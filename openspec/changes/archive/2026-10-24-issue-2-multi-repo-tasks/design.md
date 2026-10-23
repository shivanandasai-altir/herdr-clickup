# Design: a task folder with a worktree per repo

## Context

A herdr worktree workspace holds one checkout. The plugin already links a pane to a task. Repos open
in herdr are the choices for a start.

## Decisions

- **A task folder, not the user's parent folder.** It keeps the user's checkouts untouched and two
  tasks cannot collide on one checkout. The folder is not a git repo, so herdr shows no branch for
  that workspace; the repos inside work normally.
- **Git directly inside the task folder.** `herdr worktree create` always opens a workspace, and a
  multi-repo task needs one workspace for all repos. Single-repo starts keep `herdr worktree create`.
- **One branch name in every repo.** `CU-<id>-<slug>` makes the PRs easy to link to each other and
  to the ticket. Each repo starts from `origin/HEAD`, or its current HEAD when it has no remote.
- **Generated AGENTS.md.** The agent gets the repo list and the one-PR-per-repo rule. Claude also
  gets `CLAUDE.md`, which it reads instead.
- **`repos.txt` is the approval boundary.** A spec start writes every selected repo. The user edits
  the list during review. Implementation reads it, adds the missing worktrees, then sends the apply
  workflow. No ClickUp write is involved.
- **The first picked repo hosts the spec.** A spec start needs one worktree to hold the OpenSpec
  change. The other repos stay at their own paths. The prompt lets the agent read them to judge
  impact, but not change them, and implementation adds their worktrees before any code work.
- **OpenSpec runs in the host worktree.** The agent's cwd is the task folder, which is not an
  OpenSpec project. Both the spec and apply prompts name the host repo and tell the agent to run
  every `openspec` command there.
- **Close asks once and forces.** The modal names the repos and the dirty ones. A second keypress
  removes the worktrees with `git worktree remove --force`, removes the task folder, and closes the
  workspace.

## Risks

- `git worktree add -b` fails when the branch already exists. Reopening an existing task is #16.
- A failed start can leave a partial task folder. Cleanup is #15.
- The task folder is not a git repo, so herdr cannot show a branch in its sidebar for that workspace.
