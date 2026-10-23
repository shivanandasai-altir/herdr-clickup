## ADDED Requirements

### Requirement: A drafted update can be edited before approval
The update dialog SHALL open the draft in `$EDITOR` on `e`, with the comment and a `status:` line.
Saving SHALL return to the dialog with the edited draft and SHALL save it on the link. The draft is
still posted only on Enter.

#### Scenario: Edit the comment and the status
- **WHEN** the user presses `e` on a drafted update and saves the editor with new text
- **THEN** the dialog shows the new comment and status
- **AND** the link stores the edited draft

#### Scenario: Empty the comment
- **WHEN** the edited comment is empty and the status line names a status
- **THEN** approval posts only the status and no comment

#### Scenario: Leave and return
- **WHEN** the user edits a draft and closes the dialog with Esc
- **THEN** pressing `u` again shows the edited draft

#### Scenario: The editor fails or cancels
- **WHEN** the editor exits with a failure or the draft file cannot be read
- **THEN** the original draft stays
