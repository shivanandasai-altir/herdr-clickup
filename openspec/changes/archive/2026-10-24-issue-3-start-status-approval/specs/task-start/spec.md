## ADDED Requirements

### Requirement: A start drafts its status change
A start SHALL NOT write a ClickUp status. When a start status is configured, the start SHALL put a
status-only draft on the task link. The plugin posts the draft only after the user approves it.

#### Scenario: A code start with a status
- **WHEN** a code start succeeds and `CLICKUP_STATUS_STARTED` is set
- **THEN** the task shows a drafted update with that status and no comment
- **AND** ClickUp keeps its current status until the user approves the draft

#### Scenario: A spec start with a status
- **WHEN** a spec start succeeds and `CLICKUP_STATUS_SPEC` is set
- **THEN** the task shows the same kind of status-only draft

#### Scenario: Approval of a status-only draft
- **WHEN** the user approves a draft whose comment is empty
- **THEN** the plugin sets the status
- **AND** it posts no empty comment

#### Scenario: No status configured
- **WHEN** the start status setting is empty or absent
- **THEN** the start drafts nothing
