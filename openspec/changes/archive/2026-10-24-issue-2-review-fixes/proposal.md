# Proposal: fix the multi-repo review findings

## Why

Review found three ways a multi-repo task can do the wrong thing and four smaller gaps. A moved
cursor started the first repo instead of the highlighted one. A single-repo spec wrote a repo list
that implementation ignored. An old link hid uncommitted work before a forced close.

## What Changes

- Enter starts the highlighted repo when no repo is marked. Space marks one or more repos.
- A spec start writes `repos.txt` and the repo-list prompt only for two or more repos. A single-repo
  spec keeps today's worktree and behaviour.
- The close dialog checks the checkout folder of links written before worktree paths were stored.
- Implementation records each new worktree at once, so a later failure leaves nothing untracked.
- The picker and clicked-task pane ask for one or more repos, and the CLI takes `repo[,repo]`.
- The add-repo picker uses the same canonical path comparison as worktree creation.
- Close tolerates a task folder that a user removed by hand, and prunes stale worktree records.

## Capabilities

### Modified Capabilities

- `multi-repo-tasks`: the repo list belongs to 2+ repo spec starts, and close checks old links.

## Impact

`src/app.rs`, `src/board.rs`, `src/main.rs`, `src/taskspace.rs`, `tests/`.
