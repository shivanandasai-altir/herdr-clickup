# Tasks

## 1. Task folder

- [x] 1.1 Add `taskspace`: task dir, repo names, worktree add/remove, dirty check, AGENTS.md, repos.txt.
- [x] 1.2 Test worktree creation, branch, dirty detection, removal and repo-list parsing with real git repos.

## 2. Start

- [x] 2.1 Take a repo list in `app::start`; keep the single-worktree path for one repo.
- [x] 2.2 Create the task folder, worktrees, instructions and one workspace for 2+ repos.
- [x] 2.3 Store the folder, repos, title, URL and agent on the link.

## 3. Spec and implement

- [x] 3.1 Write `repos.txt` at a spec start and ask the agent to review it.
- [x] 3.2 Add the approved repos' worktrees at `i`, then send the apply workflow.

## 4. Board

- [x] 4.1 Make the repo step a multi-select; first pick hosts the spec.
- [x] 4.2 Show branch, repos and task folder in the detail view.
- [x] 4.3 Add a repo (`A`) and close a task (`X`) with a dirty warning.
- [x] 4.4 Test the multi-select result and the task-folder helpers.

## 5. Verify

- [x] 5.1 `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`.
- [x] 5.2 `openspec validate issue-2-multi-repo-tasks --strict`.
