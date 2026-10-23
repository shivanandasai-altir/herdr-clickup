# Design: keep the old single-repo promise

## Decisions

- **The highlight is the default.** No repo is pre-marked. Enter starts the highlighted repo, which
  is the old single-repo key path. Space marks repos for a multi-repo start. This removes the
  "first pick" that started repo 1 after the cursor moved to repo 2.
- **A single-repo spec has no repo list.** It uses the old worktree, and implementation cannot add
  repos. Writing a list that implementation ignores was worse than no list.
- **Each worktree is recorded as soon as it exists.** Implementation adds one repo at a time and
  stores it on the link. A failure on repo 3 leaves repos 1 and 2 known to close.
- **Close checks the old checkout folder.** A link with no `worktrees` still has `dir`. The preview
  runs `git status` there, so the forced close warning is honest.
- **Close treats a missing folder as closed, then prunes.** A folder removed by hand no longer blocks
  the link forever. `git worktree prune` clears records left by a partial close.

## Risks

- A single-repo spec cannot gain a second repo later. The user starts a multi-repo task for that.
- The CLI reads `repo[,repo]` as a list. A repo path that contains a comma is not supported there.
