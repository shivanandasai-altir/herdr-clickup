## MODIFIED Requirements

### Requirement: Implement runs the apply workflow
The implement action SHALL send the OpenSpec apply workflow to the agent in the task's existing pane
only when the change passes strict validation and the agent is idle. It SHALL claim the link once,
inside the state lock, and SHALL move it to implementing.

#### Scenario: Implementation starts
- **WHEN** the user presses `i` on a task whose spec is drafted, validates, and whose agent is idle
- **THEN** the agent receives `openspec instructions apply --change <change>`
- **AND** the next finish reports an implementation, not a spec draft

#### Scenario: The change does not validate
- **WHEN** `openspec validate --strict` fails for the change
- **THEN** no prompt is sent
- **AND** the board shows the validation error
- **AND** the link stays spec drafted

#### Scenario: The agent is still working
- **WHEN** the user presses `i` while the spec agent is working
- **THEN** no prompt is sent
- **AND** the board asks the user to wait for the finish

#### Scenario: A second press
- **WHEN** the user presses `i` twice before the board reads the new phase
- **THEN** only the first press claims the link and sends a prompt
- **AND** the second press sends nothing
