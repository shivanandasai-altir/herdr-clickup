# multi-repo-tasks Specification

## Purpose
How one ClickUp task spans several repos: one task folder, one worktree per repo on the same
branch, one workspace, one agent, and board actions to add a repo and close the task.

## Requirements

### Requirement: Two or more repos share one task folder
A code start with two or more repos SHALL create `~/.herdr/tasks/<branch>/`, add a worktree of each repo
on the same branch, write the task instructions, and open one workspace and one agent on the folder.

#### Scenario: A multi-repo code start
- **WHEN** the user picks two or more repos for a code start
- **THEN** the task folder holds one worktree per repo on `CU-<id>-<slug>`
- **AND** one workspace and one agent run on the task folder
- **AND** `AGENTS.md` names the branch and every repo

#### Scenario: Claude reads CLAUDE.md
- **WHEN** the chosen agent is Claude
- **THEN** the task folder also holds `CLAUDE.md` with the same instructions

### Requirement: One repo keeps the single worktree start
A start with one repo SHALL behave as before: a herdr worktree workspace on `CU-<id>-<slug>`.

#### Scenario: A single-repo start
- **WHEN** the user picks one repo
- **THEN** the plugin creates the same worktree and workspace it created before this change

### Requirement: The approved repo list drives implementation
A spec start SHALL create only the worktree of the repo that hosts the change, and SHALL write the
selected repos to `repos.txt` in the change folder. The implement action SHALL add a worktree for
each repo in `repos.txt` that the task does not have yet, before it sends the apply workflow.

#### Scenario: The user adjusts the list
- **WHEN** the user edits `repos.txt` during spec review
- **THEN** implementation creates worktrees for exactly those repos on the task branch

#### Scenario: A repo is already present
- **WHEN** `repos.txt` names a repo that already has a worktree in the task folder
- **THEN** implementation does not create a second worktree for it

### Requirement: The board shows and changes a task's repos
The task detail SHALL show the branch, the repos, and the task folder of a multi-repo task. The
board SHALL add a repo to a running task, and SHALL close a task after one confirmation that lists
repos with uncommitted changes.

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
