# Proposal: add an MIT license

## Why

The repo is public but has no license, so nobody may legally reuse it. The other herdr plugins
(herdr-projects, reviewr, herdr-openspec) use MIT.

## What Changes

- Add `LICENSE` with the MIT text.
- Set `license = "MIT"` in `Cargo.toml`.
- Add a License section to the README.

No runtime behaviour changes.

## Impact

`LICENSE`, `Cargo.toml`, `README.md`.
