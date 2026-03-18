# Contributing to myx

Thanks for contributing to `myx`. This project is building a portability + trust layer for agent capabilities, so correctness and predictability are more important than shipping fast but ambiguous behavior.

## Core Development Rules

These rules apply to maintainers and external contributors equally:

1. Specs are source of truth for behavior.
2. Determinism is required for install/build flows.
3. Permission enforcement must be explicit and testable.
4. Any behavior change must include docs/spec updates in the same PR.

## MVP Contract Guardrails

For MVP work, `rfcs/0004-cli-contract.md` is the normative behavior contract.

Required guardrails:

1. No MVP behavior change without RFC delta in the same PR.
2. If a change is out of MVP scope, move it to `rfcs/0005-post-mvp-expansion.md` instead of silently extending MVP.
3. Implementation, schemas, and examples must be updated together when a contract changes.
4. Every contract change must include tests for determinism, policy behavior, or artifact output.

Contract checklist for behavior changes:

- command/flag semantics
- exit codes
- JSON output shape
- artifact paths and filenames
- schema compatibility
- lockfile and policy behavior

## What to Contribute

- Rust core crates in `crates/`
- Schemas in `schemas/`
- RFCs/spec docs in `rfcs/` and `docs/`
- Examples and fixtures in `examples/`

## Local Setup

1. Install stable Rust (`rustup`).
2. Clone the repo.
3. Run:

```bash
cargo check
```

## Development Workflow

1. Create a branch from `main`.
2. Keep PRs focused on one behavior change.
3. If behavior changes, update the relevant RFC or spec doc.
4. Run required checks before opening a PR.

## Required Checks

Run all of these locally before submitting:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If your change touches schemas or adapters, include fixture/golden tests that prove deterministic output.

## Specs and RFC Expectations

- Public behavior contracts belong in RFCs.
- If implementation and RFC conflict, update one so they match before merge.
- New schema versions must include migration notes and compatibility impact.

## Security and Policy Changes

Changes affecting any of these must include tests and rationale:

- network allowlists
- subprocess execution rules
- filesystem access boundaries
- lockfile integrity behavior

Do not weaken policy enforcement without explicit maintainer approval and documented tradeoffs.

## Commit and PR Guidelines

- Use clear, imperative commit messages.
- PR description must include:
  - what changed
  - why it changed
  - how it was tested
  - any RFC/doc updates
- Link related issue/RFC when available.

## Agent and Automation Contributions

Any AI or automation-assisted contribution must follow `AGENTS.md` in the repo root.
If agent-generated code changes behavior without matching RFC/schema updates, the PR should not be merged.

## License

By contributing, you agree your contributions are licensed under the repository license (`Apache-2.0`).
