## MODIFIED Requirements

### Requirement: The approved repo list drives implementation
A spec start with two or more repos SHALL create only the worktree of the repo that hosts the change,
and SHALL write the selected repos to `repos.txt` in the change folder. A single-repo spec start SHALL
keep the single worktree and SHALL NOT write `repos.txt`. The implement action SHALL add a worktree
for each repo in `repos.txt` that the task does not have yet, before it sends the apply workflow.

#### Scenario: The user adjusts the list
- **WHEN** the user edits `repos.txt` during the review of a multi-repo spec
- **THEN** implementation creates worktrees for exactly those repos on the task branch

#### Scenario: A repo is already present
- **WHEN** `repos.txt` names a repo that already has a worktree in the task folder
- **THEN** implementation does not create a second worktree for it

#### Scenario: A single-repo spec start
- **WHEN** the user starts a spec with one repo
- **THEN** the start matches the single-worktree start from before
- **AND** no `repos.txt` is written

### Requirement: The board shows and changes a task's repos
The task detail SHALL show the branch, the repos, and the task folder of a multi-repo task. The
board SHALL add a repo to a running multi-repo task, and SHALL close a task after one confirmation
that lists repos with uncommitted changes.

#### Scenario: Add a repo
- **WHEN** the user presses `A` on a running multi-repo task and picks a repo
- **THEN** a worktree of that repo joins the task folder on the task branch
- **AND** `AGENTS.md` and an existing `repos.txt` include it

#### Scenario: Close a task
- **WHEN** the user presses `X` and then Enter on the close dialog
- **THEN** every worktree and the task folder are removed
- **AND** the workspace is closed and the pane link is forgotten

#### Scenario: Uncommitted work
- **WHEN** a worktree has uncommitted or untracked changes
- **THEN** the close dialog names that repo before the user confirms

#### Scenario: A link from before worktrees were stored
- **WHEN** the task has a checkout folder but no stored worktree list
- **THEN** the close dialog checks that folder for uncommitted changes

#### Scenario: A folder already gone
- **WHEN** the user removed the task folder by hand
- **THEN** close removes the link and the stale worktree records without an error
