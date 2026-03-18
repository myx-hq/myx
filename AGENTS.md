# Agent Rules (MVP)

This file defines mandatory rules for AI/automation agents contributing to `myx`.

## Authority and Scope

1. Treat `rfcs/0004-cli-contract.md` as the MVP source of truth.
2. Do not silently extend MVP scope.
3. Any non-MVP behavior must be recorded in `rfcs/0005-post-mvp-expansion.md`.

## Required Change Discipline

For any behavior change, agents must update all relevant contract surfaces in the same change:

- RFC text (`rfcs/`)
- docs (`docs/`)
- schemas (`schemas/`) when shape changes
- examples/fixtures (`examples/`) when applicable
- tests

If behavior changes but contract docs are not updated, stop and update specs before finalizing.

## MVP Invariants

Agents must preserve these MVP invariants unless explicitly changing RFC 0004:

1. Command surface: `init`, `add`, `inspect`, `build`, `run`.
2. Tier-1 targets: `openai`, `mcp`, `skill`.
3. Deterministic lockfile and build outputs.
4. Enforcement-grade policy checks for install/runtime.
5. Subprocess execution constraints:
- exact command allowlist
- explicit cwd allowlist
- explicit env passthrough allowlist
- required timeout
- no shell invocation

## PR Requirements for Agent Changes

Agent-generated PRs must include:

1. Explicit statement of behavior changes.
2. RFC/spec/schema references updated.
3. Test evidence (`cargo test --workspace` at minimum).
4. Contract checklist coverage:
- exit codes
- JSON output shape
- artifact outputs
- policy behavior
