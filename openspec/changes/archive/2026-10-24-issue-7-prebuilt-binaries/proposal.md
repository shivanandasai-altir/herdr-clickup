# Proposal: ship prebuilt binaries

## Why

`herdr plugin install` runs `cargo build --release` on the user's machine (`bin/build.sh`), so every
user needs a Rust toolchain and a multi-minute build. herdr-projects and reviewr download a prebuilt
binary for the platform from the GitHub release that matches the version.

## What Changes

- A release workflow on `v*` tags builds macOS (arm64, x86_64) and Linux (x86_64, arm64) binaries and
  attaches them to the release as `herdr-clickup-<target>`.
- `bin/build.sh` maps the machine to a target and downloads that asset. It builds from source only
  when the asset is missing, or when a newer local build already exists.
- `bin/run.sh` builds on demand if the binary is missing.
- The README says the install no longer needs Rust when an asset exists.

No runtime behaviour changes.

## Impact

`.github/workflows/release.yml` (new), `bin/build.sh`, `bin/run.sh`, `herdr-plugin.toml`, `README.md`.
