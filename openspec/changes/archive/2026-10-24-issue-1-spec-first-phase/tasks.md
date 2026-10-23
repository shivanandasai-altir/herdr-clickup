# Tasks

## 1. State

- [x] 1.1 Add the `Phase` enum and an optional phase on `Link`, with a fallback for old links.
- [x] 1.2 Set the phase at start, advance it after a report, and reset it for implementation.

## 2. Implement

- [x] 2.1 Add the apply prompt and `app::implement`, which claims the next finish before prompting.
- [x] 2.2 Draft the implementation report with the done status, and the spec report with the spec status.

## 3. Board

- [x] 3.1 Show a PHASE column from the state file.
- [x] 3.2 Offer `i` only for a drafted spec, in the task table and the detail view.

## 4. Verify

- [x] 4.1 Test phase fallback, phase advance, both report drafts and the apply prompt.
- [x] 4.2 `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- [x] 4.3 `openspec validate issue-1-spec-first-phase --strict`.
