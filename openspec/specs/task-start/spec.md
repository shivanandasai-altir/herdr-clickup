# task-start Specification

## Purpose
How a ClickUp task becomes an OpenSpec change, is reviewed, is implemented, and reports back to
ClickUp. The plugin starts the agent on the task, tracks the task's phase, and drafts every ClickUp
write for approval.

## Requirements

### Requirement: Spec-first start writes a change and no code
A spec-first start SHALL create the task worktree, create the OpenSpec change there, write the whole
ClickUp ticket into the change folder, and prompt the agent to write the proposal, specs, design and
tasks only.

#### Scenario: A spec start
- **WHEN** the user starts a task in spec mode
- **THEN** the worktree holds an OpenSpec change seeded with the ticket
- **AND** the first prompt forbids application code and asks for a validated change

#### Scenario: The whole ticket is in the prompt
- **WHEN** a task has checklists, subtasks, attachments, comments, custom fields or a parent task
- **THEN** the prompt or the seeded ticket file includes them

### Requirement: The board shows the task phase
The board SHALL show each linked task's phase, and SHALL offer the implement action only when a
drafted change waits for implementation.

#### Scenario: A drafted spec
- **WHEN** the spec agent finishes and the link moves to spec drafted
- **THEN** the board shows `spec ✓` for the task
- **AND** the `i` action is offered for that task

#### Scenario: No change yet
- **WHEN** a task has no link or no OpenSpec change
- **THEN** the `i` action is not offered
- **AND** pressing `i` explains that the task needs a spec first

### Requirement: Implement runs the apply workflow
The implement action SHALL send the OpenSpec apply workflow to the agent in the task's existing pane
and SHALL move the link to implementing.

#### Scenario: Implementation starts
- **WHEN** the user presses `i` on a task whose spec is drafted
- **THEN** the agent receives `openspec instructions apply --change <change>`
- **AND** the next finish reports an implementation, not a spec draft

### Requirement: Reports separate spec drafting from implementation
The hook SHALL draft a spec report for a finished spec phase, and an implementation report for a
finished implementation phase. Neither report is posted without approval.

#### Scenario: Spec finish
- **WHEN** the agent finishes while the link is spec drafting
- **THEN** the draft says the spec is ready for review
- **AND** it uses the spec status when one is configured

#### Scenario: Implementation finish
- **WHEN** the agent finishes while the link is implementing
- **THEN** the draft says the implementation is ready for review
- **AND** it uses the done status when one is configured
