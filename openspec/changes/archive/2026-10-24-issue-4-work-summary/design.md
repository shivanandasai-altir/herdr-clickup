# Design: facts from the worktree

## Decisions

- **Read the worktree, not ClickUp.** Commits, the diff stat and the proposal are local. No extra API
  call, and the summary describes the exact checkout the agent worked in.
- **Build the summary outside the state lock.** It spawns git up to three times per repo. The hook
  reads the link, builds the summary, then takes the lock to decide and claim.
- **Base the diff on the remote default branch.** `origin/HEAD` gives the branch point. A repo with
  no remote falls back to the last five commits, the working tree, then a count of changed files.
- **Cap the text.** Up to 10 commits, a 500-character Why, and a one-line diff stat keep the ClickUp
  comment readable.
- **The head stays.** The existing sentence ("Implementation done...", "Spec drafted...") stays
  first, and the summary follows after a blank line. A reader who skims gets both.

## Risks

- A large branch with many commits shows only the first 10. The diff stat still shows the size.
- The summary is a snapshot at the finish event. Work after that is not in the draft.
