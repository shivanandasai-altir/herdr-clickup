# Design: one claim, one validated change

## Decisions

- **Validate before the claim.** If a change does not validate, no prompt is sent and the link keeps
  its state, so the user can fix the spec and press `i` again.
- **Claim under the state lock.** The claim checks `can_implement` and sets the phase in one
  `store::update`. Two board presses, or a hook event at the same time, cannot both win.
- **Refuse while working.** The board explains why. A prompt that queues behind a running turn would
  make that turn's finish look like an implementation.
- **Store the worktree folder on the link.** Validation needs a working directory. Later multi-repo
  starts use the same field for the task folder.

## Risks

- Validation adds a short subprocess call before the prompt. The board already runs the action on a
  thread, so the UI does not block.
