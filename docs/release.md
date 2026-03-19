# Release Guide

This document defines the MVP release process for `myx` from this repository.

## Scope

- Build and publish macOS binaries (`aarch64-apple-darwin`, `x86_64-apple-darwin`).
- Publish GitHub release artifacts from a `v*` tag.
- Generate Homebrew formula handoff content for `myx-hq/homebrew-myx`.

## Pre-Release Checklist

Run from `main` with a clean working tree:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-mvp-contract.sh
./scripts/smoke-mvp-loop.sh
./scripts/benchmark-warm-cache-add.sh --out /tmp/warm-cache-add.json
```

Confirm MVP contract and RFCs remain aligned before tagging.

## Cut a Release

1. Pick version `X.Y.Z` (matches workspace version semantics).
2. Create and push tag:

```bash
git checkout main
git pull --ff-only
git tag vX.Y.Z
git push origin vX.Y.Z
```

3. Tag push triggers `.github/workflows/release.yml`, which:
- builds release binaries for both macOS targets
- creates `myx-<version>-<target>.tar.gz` artifacts
- emits per-archive and consolidated SHA256 files
- publishes a GitHub release with all artifacts
- generates `homebrew-myx.rb` handoff artifact

## Homebrew Tap Update

`myx-hq/myx` does not own the tap formula repository. After release completes:

1. Download `homebrew-myx.rb` from release artifacts.
2. Open PR in `myx-hq/homebrew-myx` updating `Formula/myx.rb`.
3. Merge formula PR.
4. Verify install:

```bash
brew update
brew tap myx-hq/myx
brew install myx
myx --help
```

## Dry-Run Packaging

Use manual workflow dispatch (`Release`) with a `version` input to validate packaging logic without creating a GitHub release.
