## ADDED Requirements

### Requirement: A finish draft summarises the work
A finish draft SHALL say what the agent did. A code finish SHALL list the branch's commits and the
changed-file summary. A spec finish SHALL include the proposal's Why and the task count. A draft is
still posted only after the user approves it.

#### Scenario: A code finish
- **WHEN** a code agent finishes on a branch with commits and changed files
- **THEN** the drafted comment lists the commits and the diff stat
- **AND** the status change stays in the same draft

#### Scenario: A multi-repo finish
- **WHEN** the task has worktrees in several repos
- **THEN** the draft groups the commits and changed files by repo

#### Scenario: A spec finish
- **WHEN** a spec agent finishes
- **THEN** the drafted comment includes the proposal's Why and "N of M" tasks done

#### Scenario: No summaries available
- **WHEN** the worktree or the proposal files cannot be read
- **THEN** the draft keeps the head sentence and posts nothing by itself
